use crate::uar::domain::skills::Skill;
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
pub enum MatchReason {
    ExplicitTag,
    VectorSimilarity(f32),
    LLMSelected { reasoning: String },
}

#[derive(Debug, Clone)]
pub struct SkillMatch {
    pub skill_id: String,
    pub score: f32,
    pub reason: MatchReason,
    pub skill: Skill,
}

#[async_trait]
pub trait SkillMatcher: Send + Sync {
    async fn match_skills(
        &self,
        query: &str,
        registry: &SkillRegistry,
    ) -> anyhow::Result<Vec<SkillMatch>>;
}
