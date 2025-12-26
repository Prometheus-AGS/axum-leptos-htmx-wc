use crate::uar::domain::matching::{MatchReason, SkillMatch, SkillMatcher};
use crate::uar::runtime::skills::SkillRegistry;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct TagMatcher;

impl TagMatcher {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SkillMatcher for TagMatcher {
    async fn match_skills(&self, query: &str, registry: &SkillRegistry) -> Result<Vec<SkillMatch>> {
        let skills = registry.list();
        let mut matches = Vec::new();
        let lower_query = query.to_lowercase();

        for skill in skills {
            // Check triggers
            for keyword in &skill.triggers.keywords {
                let lower_kw = keyword.to_lowercase();
                // Simple inclusion or exact? Let's do inclusion logic for now but specific enough
                // Or maybe we treat triggers as "tags"

                if lower_query.contains(&lower_kw) {
                    matches.push(SkillMatch {
                        skill_id: skill.skill_id.clone(),
                        score: 1.0, // High confidence
                        reason: MatchReason::ExplicitTag,
                        skill: skill.clone(),
                    });
                    break;
                }
            }
        }

        Ok(matches)
    }
}
