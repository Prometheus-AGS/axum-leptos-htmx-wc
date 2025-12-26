use crate::session::Session;
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
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

    // =========================================================================
    // Knowledge Base Management
    // =========================================================================

    /// Save or update a knowledge base definition.
    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()>;

    /// Get a knowledge base by ID.
    async fn get_knowledge_base(&self, id: &str) -> Result<Option<KnowledgeBase>>;

    /// Get a knowledge base by name.
    async fn get_knowledge_base_by_name(&self, name: &str) -> Result<Option<KnowledgeBase>>;

    /// List all knowledge bases.
    async fn list_knowledge_bases(&self) -> Result<Vec<KnowledgeBase>>;

    /// Delete a knowledge base and all its chunks/documents.
    async fn delete_knowledge_base(&self, id: &str) -> Result<()>;

    // =========================================================================
    // Knowledge Chunk Management
    // =========================================================================

    /// Save a knowledge chunk.
    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()>;

    /// Search knowledge across ALL knowledge bases (original behavior).
    async fn search_knowledge(
        &self,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>>;

    /// Search knowledge scoped to specific knowledge base IDs.
    async fn search_knowledge_scoped(
        &self,
        kb_ids: &[&str],
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>>;

    // =========================================================================
    // Document Tracking
    // =========================================================================

    /// Save a document record.
    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()>;

    /// Get a document by ID.
    async fn get_document(&self, id: &str) -> Result<Option<KnowledgeDocument>>;

    /// List documents in a knowledge base.
    async fn list_documents(&self, kb_id: &str) -> Result<Vec<KnowledgeDocument>>;

    /// Update document processing status.
    async fn update_document_status(&self, doc_id: &str, status: &DocumentStatus) -> Result<()>;

    /// Delete a document and all its associated chunks.
    async fn delete_document(&self, doc_id: &str) -> Result<()>;

    // =========================================================================
    // Agent Persistence
    // =========================================================================

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

    // =========================================================================
    // Memory System
    // =========================================================================

    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()>;
    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>>;
}
