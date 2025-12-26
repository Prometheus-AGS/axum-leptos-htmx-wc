// Use String for ISO8601/RFC3339 to avoid chrono serde feature dominance issues

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: String,
    pub config: KbConfig,
    pub created_at: String, // RFC3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbConfig {
    pub embedding_model: String,
    pub chunk_strategy: crate::uar::rag::chunking::ChunkingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: Uuid,
    pub kb_id: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    // Embedding is not typically serialized to frontend, but good to have
    #[serde(skip)]
    pub embedding: Vec<f32>,
    pub created_at: String, // RFC3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMatch {
    pub chunk: KnowledgeChunk,
    pub score: f32,
}
