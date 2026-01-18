//! TF-IDF based intent classifier.
//!
//! Pure Rust implementation with no external dependencies.
//! Builds vocabulary/IDF from skill metadata at startup.

use super::{ClassificationResult, IntentClassifier, IntentScore};
use crate::uar::domain::skills::Skill;
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// TF-IDF classifier for intent matching.
///
/// Builds an inverted index from skill metadata (title, description, keywords)
/// and uses cosine similarity to rank skills against queries.
#[derive(Debug)]
pub struct TfIdfClassifier {
    /// Number of top candidates to return
    topk: usize,
    /// Internal index state
    index: RwLock<TfIdfIndex>,
}

/// Internal TF-IDF index.
#[derive(Debug, Default)]
struct TfIdfIndex {
    /// Document frequency for each term
    df: HashMap<String, usize>,
    /// Total number of documents (skills)
    num_docs: usize,
    /// TF-IDF vectors for each skill (skill_id -> term -> tf-idf score)
    skill_vectors: HashMap<String, HashMap<String, f32>>,
    /// Skill data for returning with results
    skills: HashMap<String, Skill>,
}

impl TfIdfClassifier {
    /// Creates a new TF-IDF classifier.
    pub fn new(topk: usize) -> Self {
        Self {
            topk,
            index: RwLock::new(TfIdfIndex::default()),
        }
    }

    /// Tokenizes text into lowercase terms.
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() >= 2) // Skip single-char tokens
            .map(String::from)
            .collect()
    }

    /// Computes term frequency for a list of tokens.
    fn compute_tf(tokens: &[String]) -> HashMap<String, f32> {
        let mut tf: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *tf.entry(token.clone()).or_default() += 1;
        }

        let max_freq = tf.values().copied().max().unwrap_or(1) as f32;

        tf.into_iter()
            .map(|(term, count)| {
                // Augmented frequency to prevent bias towards longer documents
                let normalized = 0.5 + 0.5 * (count as f32 / max_freq);
                (term, normalized)
            })
            .collect()
    }

    /// Computes IDF for a term.
    fn compute_idf(df: usize, num_docs: usize) -> f32 {
        if df == 0 || num_docs == 0 {
            return 0.0;
        }
        // Smooth IDF to avoid division by zero and extreme values
        ((num_docs as f32 + 1.0) / (df as f32 + 1.0)).ln() + 1.0
    }

    /// Computes cosine similarity between two sparse vectors.
    fn cosine_similarity(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
        let dot: f32 = a
            .iter()
            .filter_map(|(term, a_val)| b.get(term).map(|b_val| a_val * b_val))
            .sum();

        let norm_a: f32 = a.values().map(|v| v * v).sum::<f32>().sqrt();
        let norm_b: f32 = b.values().map(|v| v * v).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }

    /// Builds the index from a skill registry.
    async fn build_index(&self, registry: &SkillRegistry) {
        let skills = registry.list();
        let num_docs = skills.len();

        if num_docs == 0 {
            let mut index = self.index.write().await;
            *index = TfIdfIndex::default();
            return;
        }

        // First pass: compute document frequencies
        let mut df: HashMap<String, usize> = HashMap::new();
        let mut skill_tokens: Vec<(Skill, Vec<String>)> = Vec::with_capacity(num_docs);

        for skill in skills {
            // Combine title, description, and keywords into document text
            let mut doc_text = format!("{} {} ", skill.title, skill.description);
            for kw in &skill.triggers.keywords {
                doc_text.push_str(kw);
                doc_text.push(' ');
            }

            let tokens = Self::tokenize(&doc_text);

            // Count unique terms for DF
            let unique_terms: std::collections::HashSet<_> = tokens.iter().cloned().collect();
            for term in unique_terms {
                *df.entry(term).or_default() += 1;
            }

            skill_tokens.push((skill, tokens));
        }

        // Second pass: compute TF-IDF vectors
        let mut skill_vectors: HashMap<String, HashMap<String, f32>> = HashMap::new();
        let mut skills_map: HashMap<String, Skill> = HashMap::new();

        for (skill, tokens) in skill_tokens {
            let tf = Self::compute_tf(&tokens);

            let tfidf: HashMap<String, f32> = tf
                .into_iter()
                .map(|(term, tf_val)| {
                    let idf = Self::compute_idf(*df.get(&term).unwrap_or(&0), num_docs);
                    (term, tf_val * idf)
                })
                .collect();

            skill_vectors.insert(skill.skill_id.clone(), tfidf);
            skills_map.insert(skill.skill_id.clone(), skill);
        }

        let mut index = self.index.write().await;
        *index = TfIdfIndex {
            df,
            num_docs,
            skill_vectors,
            skills: skills_map,
        };
    }
}

#[async_trait]
impl IntentClassifier for TfIdfClassifier {
    async fn classify(
        &self,
        query: &str,
        _context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        // Ensure index is built
        {
            let index = self.index.read().await;
            if index.num_docs == 0 && !registry.list().is_empty() {
                drop(index);
                self.build_index(registry).await;
            }
        }

        let index = self.index.read().await;

        if index.num_docs == 0 {
            return Ok(ClassificationResult::new(Vec::new()));
        }

        // Tokenize query and compute TF-IDF
        let query_tokens = Self::tokenize(query);
        let query_tf = Self::compute_tf(&query_tokens);

        let query_tfidf: HashMap<String, f32> = query_tf
            .into_iter()
            .map(|(term, tf_val)| {
                let idf = Self::compute_idf(*index.df.get(&term).unwrap_or(&0), index.num_docs);
                (term, tf_val * idf)
            })
            .collect();

        // Compute similarity with each skill
        let mut scores: Vec<IntentScore> = index
            .skill_vectors
            .iter()
            .map(|(skill_id, skill_vec)| {
                let similarity = Self::cosine_similarity(&query_tfidf, skill_vec);
                IntentScore {
                    label: skill_id.clone(),
                    score: similarity,
                    skill: index.skills.get(skill_id).cloned(),
                }
            })
            .filter(|s| s.score > 0.0) // Only include non-zero matches
            .collect();

        // Sort by score descending and take top-k
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(self.topk);

        Ok(ClassificationResult::new(scores))
    }

    async fn rebuild_index(&self, registry: &SkillRegistry) -> anyhow::Result<()> {
        self.build_index(registry).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::domain::skills::{SkillConstraints, SkillTriggers};

    fn make_test_skill(id: &str, title: &str, description: &str, keywords: Vec<&str>) -> Skill {
        Skill {
            skill_id: id.to_string(),
            version: "1.0.0".to_string(),
            title: title.to_string(),
            description: description.to_string(),
            triggers: SkillTriggers {
                keywords: keywords.into_iter().map(String::from).collect(),
                semantic: None,
            },
            prompt_overlay: String::new(),
            preferred_tools: Vec::new(),
            mcp_config: None,
            constraints: SkillConstraints::default(),
        }
    }

    #[tokio::test]
    async fn test_tfidf_classifier_basic() {
        let classifier = TfIdfClassifier::new(5);
        let mut registry = SkillRegistry::default();

        registry
            .register(make_test_skill(
                "db-helper",
                "Database Helper",
                "Helps with PostgreSQL and SQL queries",
                vec!["postgres", "sql", "database"],
            ))
            .await;

        registry
            .register(make_test_skill(
                "weather",
                "Weather Service",
                "Provides weather forecasts and conditions",
                vec!["weather", "forecast", "temperature"],
            ))
            .await;

        let result = classifier
            .classify("help me write a SQL query for postgres", &[], &registry)
            .await
            .unwrap();

        assert!(!result.out_of_scope);
        assert!(!result.scores.is_empty());
        // The DB skill should rank higher
        assert_eq!(result.scores[0].label, "db-helper");
    }

    #[tokio::test]
    async fn test_tfidf_classifier_no_match() {
        let classifier = TfIdfClassifier::new(5);
        let mut registry = SkillRegistry::default();

        registry
            .register(make_test_skill(
                "db-helper",
                "Database Helper",
                "Helps with PostgreSQL queries",
                vec!["postgres", "sql"],
            ))
            .await;

        let result = classifier
            .classify("xyz123 completely unrelated", &[], &registry)
            .await
            .unwrap();

        // No matching terms, should have empty results
        assert!(result.scores.is_empty() || result.scores[0].score < 0.1);
    }

    #[test]
    fn test_tokenize() {
        let tokens = TfIdfClassifier::tokenize("Hello, World! This is a TEST.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        // Single char 'a' should be filtered out
        assert!(!tokens.contains(&"a".to_string()));
    }

    #[test]
    fn test_cosine_similarity() {
        let mut a = HashMap::new();
        a.insert("hello".to_string(), 1.0);
        a.insert("world".to_string(), 1.0);

        let mut b = HashMap::new();
        b.insert("hello".to_string(), 1.0);
        b.insert("world".to_string(), 1.0);

        let sim = TfIdfClassifier::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001); // Should be 1.0 for identical vectors

        let mut c = HashMap::new();
        c.insert("foo".to_string(), 1.0);
        c.insert("bar".to_string(), 1.0);

        let sim2 = TfIdfClassifier::cosine_similarity(&a, &c);
        assert!(sim2.abs() < 0.001); // Should be 0.0 for orthogonal vectors
    }
}
