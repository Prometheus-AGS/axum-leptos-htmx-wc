//! Intent classification module for skill matching.
//!
//! Provides a pluggable abstraction for classifying user queries into intents,
//! which are then matched to skills. Supports multiple backends:
//! - Rules-based (keyword/tag matching)
//! - TF-IDF (pure Rust, no external dependencies)
//! - WASM (for non-Rust classifiers like spaCy/BERT)

pub mod rules;
pub mod tfidf;
pub mod wasm;

use crate::uar::domain::skills::Skill;
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Score for an intent/skill classification result.
#[derive(Debug, Clone)]
pub struct IntentScore {
    /// The skill ID or intent label
    pub label: String,
    /// Confidence score (0.0 to 1.0)
    pub score: f32,
    /// The matched skill (if available)
    pub skill: Option<Skill>,
}

impl PartialEq for IntentScore {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && (self.score - other.score).abs() < f32::EPSILON
    }
}

impl IntentScore {
    /// Creates a new intent score.
    pub fn new(label: impl Into<String>, score: f32) -> Self {
        Self {
            label: label.into(),
            score,
            skill: None,
        }
    }

    /// Creates a new intent score with an associated skill.
    pub fn with_skill(label: impl Into<String>, score: f32, skill: Skill) -> Self {
        Self {
            label: label.into(),
            score,
            skill: Some(skill),
        }
    }
}

/// Result of intent classification.
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Ranked list of intent scores (highest first)
    pub scores: Vec<IntentScore>,
    /// Whether the query appears to be out-of-scope (no good match)
    pub out_of_scope: bool,
}

impl ClassificationResult {
    /// Creates a new classification result.
    pub fn new(scores: Vec<IntentScore>) -> Self {
        Self {
            out_of_scope: scores.is_empty() || scores.first().is_some_and(|s| s.score < 0.1),
            scores,
        }
    }

    /// Returns the top-scoring intent, if any.
    pub fn top(&self) -> Option<&IntentScore> {
        self.scores.first()
    }

    /// Returns true if the top score meets the accept threshold and margin.
    pub fn should_accept(&self, accept_threshold: f32, margin_threshold: f32) -> bool {
        if let Some(top) = self.scores.first() {
            if top.score < accept_threshold {
                return false;
            }
            // Check margin between top-1 and top-2
            if let Some(second) = self.scores.get(1) {
                if top.score - second.score < margin_threshold {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

/// Backend type for intent classification.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClassifierBackend {
    /// Rules-based classification (keyword/tag matching)
    Rules,
    /// TF-IDF based classification (pure Rust, no external deps)
    #[default]
    Tfidf,
    /// WASM-based classification (for non-Rust classifiers)
    Wasm,
    /// Hybrid: rules first, then TF-IDF for remaining
    Hybrid,
}

/// Configuration for the intent classifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierConfig {
    /// Which backend to use
    #[serde(default)]
    pub backend: ClassifierBackend,
    /// Number of top candidates to consider
    #[serde(default = "ClassifierConfig::default_topk")]
    pub topk: usize,
    /// Minimum score to accept a classification
    #[serde(default = "ClassifierConfig::default_accept_threshold")]
    pub accept_threshold: f32,
    /// Minimum margin between top-1 and top-2 scores
    #[serde(default = "ClassifierConfig::default_margin_threshold")]
    pub margin_threshold: f32,
    /// Path to WASM component (only used when backend = wasm)
    #[serde(default)]
    pub wasm_component_path: Option<String>,
}

impl ClassifierConfig {
    fn default_topk() -> usize {
        5
    }

    fn default_accept_threshold() -> f32 {
        0.5
    }

    fn default_margin_threshold() -> f32 {
        0.05
    }
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            backend: ClassifierBackend::default(),
            topk: Self::default_topk(),
            accept_threshold: Self::default_accept_threshold(),
            margin_threshold: Self::default_margin_threshold(),
            wasm_component_path: None,
        }
    }
}

/// Trait for intent classifiers.
///
/// Implementations should classify a query and return ranked intent scores.
#[async_trait]
pub trait IntentClassifier: Send + Sync + std::fmt::Debug {
    /// Classifies the query and returns ranked intent scores.
    ///
    /// # Arguments
    /// * `query` - The user's input query
    /// * `context` - Optional conversation context (previous messages)
    /// * `registry` - The skill registry to match against
    ///
    /// # Returns
    /// A classification result with ranked scores
    async fn classify(
        &self,
        query: &str,
        context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult>;

    /// Rebuilds any internal indices (e.g., TF-IDF vocabulary).
    /// Called when skills are added/removed.
    async fn rebuild_index(&self, registry: &SkillRegistry) -> anyhow::Result<()> {
        // Default: no-op
        let _ = registry;
        Ok(())
    }
}

/// Factory for creating intent classifiers based on config.
pub fn create_classifier(config: &ClassifierConfig) -> Box<dyn IntentClassifier> {
    match config.backend {
        ClassifierBackend::Rules => Box::new(rules::RulesClassifier::new()),
        ClassifierBackend::Tfidf => Box::new(tfidf::TfIdfClassifier::new(config.topk)),
        ClassifierBackend::Wasm => {
            Box::new(wasm::WasmClassifier::new(config.wasm_component_path.clone()))
        }
        ClassifierBackend::Hybrid => Box::new(HybridClassifier::new(config.topk)),
    }
}

/// Hybrid classifier that combines rules and TF-IDF.
///
/// First checks rules for high-confidence matches, then uses TF-IDF
/// for remaining skills.
#[derive(Debug)]
pub struct HybridClassifier {
    rules: rules::RulesClassifier,
    tfidf: tfidf::TfIdfClassifier,
}

impl HybridClassifier {
    /// Creates a new hybrid classifier.
    pub fn new(topk: usize) -> Self {
        Self {
            rules: rules::RulesClassifier::new(),
            tfidf: tfidf::TfIdfClassifier::new(topk),
        }
    }
}

#[async_trait]
impl IntentClassifier for HybridClassifier {
    async fn classify(
        &self,
        query: &str,
        context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        // First, check rules
        let rules_result = self.rules.classify(query, context, registry).await?;

        // If rules found high-confidence matches, return them
        if !rules_result.out_of_scope
            && rules_result
                .scores
                .first()
                .is_some_and(|s| s.score >= 0.9)
        {
            return Ok(rules_result);
        }

        // Otherwise, use TF-IDF
        let tfidf_result = self.tfidf.classify(query, context, registry).await?;

        // Merge: rules matches get a boost
        let mut merged: Vec<IntentScore> = Vec::new();
        let rules_labels: std::collections::HashSet<_> =
            rules_result.scores.iter().map(|s| &s.label).collect();

        for mut score in tfidf_result.scores {
            if rules_labels.contains(&score.label) {
                // Boost TF-IDF score for rules matches
                score.score = (score.score + 0.3).min(1.0);
            }
            merged.push(score);
        }

        // Sort by score descending
        merged.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ClassificationResult::new(merged))
    }

    async fn rebuild_index(&self, registry: &SkillRegistry) -> anyhow::Result<()> {
        self.tfidf.rebuild_index(registry).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_result_should_accept() {
        let scores = vec![
            IntentScore::new("skill_a", 0.8),
            IntentScore::new("skill_b", 0.4),
        ];
        let result = ClassificationResult::new(scores);

        // Should accept: top score >= 0.5, margin >= 0.05
        assert!(result.should_accept(0.5, 0.05));

        // Should not accept: threshold too high
        assert!(!result.should_accept(0.9, 0.05));

        // Should accept: margin is 0.4, which is >= 0.3
        assert!(result.should_accept(0.5, 0.3));
    }

    #[test]
    fn test_classification_result_insufficient_margin() {
        let scores = vec![
            IntentScore::new("skill_a", 0.6),
            IntentScore::new("skill_b", 0.58),
        ];
        let result = ClassificationResult::new(scores);

        // Should not accept: margin is only 0.02
        assert!(!result.should_accept(0.5, 0.05));
    }

    #[test]
    fn test_classifier_config_defaults() {
        let config = ClassifierConfig::default();
        assert_eq!(config.backend, ClassifierBackend::Tfidf);
        assert_eq!(config.topk, 5);
        assert!((config.accept_threshold - 0.5).abs() < f32::EPSILON);
        assert!((config.margin_threshold - 0.05).abs() < f32::EPSILON);
    }
}
