//! Rules-based intent classifier.
//!
//! Uses keyword/tag matching to classify intents. High precision, low recall.
//! Best used as a first gate before more sophisticated classifiers.

use super::{ClassificationResult, IntentClassifier, IntentScore};
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;

/// Rules-based classifier that matches keywords and tags.
#[derive(Debug, Default)]
pub struct RulesClassifier;

impl RulesClassifier {
    /// Creates a new rules classifier.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IntentClassifier for RulesClassifier {
    async fn classify(
        &self,
        query: &str,
        _context: &[String],
        registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        let skills = registry.list();
        let mut scores = Vec::new();
        let lower_query = query.to_lowercase();

        for skill in skills {
            let mut best_score = 0.0f32;

            // Check trigger keywords
            for keyword in &skill.triggers.keywords {
                let lower_kw = keyword.to_lowercase();

                if lower_query.contains(&lower_kw) {
                    // Exact phrase match gets highest score
                    let kw_score = if lower_query == lower_kw {
                        1.0
                    } else {
                        // Partial match score based on keyword coverage
                        let coverage = lower_kw.len() as f32 / lower_query.len() as f32;
                        0.7 + (coverage * 0.3).min(0.3)
                    };
                    best_score = best_score.max(kw_score);
                }
            }

            // Check title match
            let lower_title = skill.title.to_lowercase();
            if lower_query.contains(&lower_title) {
                best_score = best_score.max(0.9);
            } else if lower_title.contains(&lower_query) {
                best_score = best_score.max(0.6);
            }

            // Only include if we found a match
            if best_score > 0.0 {
                scores.push(IntentScore::with_skill(
                    skill.skill_id.clone(),
                    best_score,
                    skill.clone(),
                ));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ClassificationResult::new(scores))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::domain::skills::{Skill, SkillConstraints, SkillTriggers};

    fn make_test_skill(id: &str, title: &str, keywords: Vec<&str>) -> Skill {
        Skill {
            skill_id: id.to_string(),
            version: "1.0.0".to_string(),
            title: title.to_string(),
            description: format!("Test skill: {title}"),
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
    async fn test_rules_classifier_keyword_match() {
        let classifier = RulesClassifier::new();
        let mut registry = SkillRegistry::default();

        let skill = make_test_skill("db-helper", "Database Helper", vec!["postgres", "sql"]);
        registry.register(skill).await;

        let result = classifier
            .classify("help me with postgres", &[], &registry)
            .await
            .unwrap();

        assert!(!result.out_of_scope);
        assert_eq!(result.scores.len(), 1);
        assert_eq!(result.scores[0].label, "db-helper");
        assert!(result.scores[0].score > 0.7);
    }

    #[tokio::test]
    async fn test_rules_classifier_no_match() {
        let classifier = RulesClassifier::new();
        let mut registry = SkillRegistry::default();

        let skill = make_test_skill("db-helper", "Database Helper", vec!["postgres", "sql"]);
        registry.register(skill).await;

        let result = classifier
            .classify("tell me about the weather", &[], &registry)
            .await
            .unwrap();

        assert!(result.out_of_scope);
        assert!(result.scores.is_empty());
    }
}
