use crate::session::Session;
use crate::uar::domain::skills::{Skill, SkillMatch};
use anyhow::Result;
use async_trait::async_trait;

pub mod providers;

#[derive(Debug)]
pub struct PostgresProvider;

#[async_trait]
pub trait PersistenceLayer: Send + Sync + std::fmt::Debug {
    // Session Management
    async fn save_session(&self, session: &Session) -> Result<()>;
    async fn load_session(&self, id: &str) -> Result<Option<Session>>;

    // Skill Management
    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()>;
    async fn search_skills(&self, query_vec: &[f32], limit: usize) -> Result<Vec<SkillMatch>>;

    // Knowledge Base Management
    async fn save_knowledge_base(
        &self,
        kb: &crate::uar::domain::knowledge::KnowledgeBase,
    ) -> Result<()>;
    async fn save_chunk(&self, chunk: &crate::uar::domain::knowledge::KnowledgeChunk)
    -> Result<()>;
    async fn search_knowledge(
        &self,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::knowledge::KnowledgeMatch>>;

    // Agent Persistence
    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()>;
    async fn load_agent(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>>;
    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>>;
    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>>;

    // Memory System
    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()>;
    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>>;
}
