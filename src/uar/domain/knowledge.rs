// Use String for ISO8601/RFC3339 to avoid chrono serde feature dominance issues

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A named knowledge base container for RAG document scoping.
/// Each KB has its own embedding model, chunking strategy, and document collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub id: String,
    /// Unique human-readable name (e.g., "default", "technical-docs")
    pub name: String,
    /// Optional description of the knowledge base
    #[serde(default)]
    pub description: Option<String>,
    pub config: KbConfig,
    pub created_at: String, // RFC3339
    pub updated_at: String, // RFC3339
}

/// Configuration for a knowledge base's processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbConfig {
    /// Embedding provider: "fastembed", "openai", "mistral"
    #[serde(default = "KbConfig::default_embedding_provider")]
    pub embedding_provider: String,
    /// Model ID for the embedding provider
    #[serde(default = "KbConfig::default_embedding_model")]
    pub embedding_model: String,
    /// Vector dimensions (None = use model default)
    #[serde(default)]
    pub vector_dimensions: Option<usize>,
    /// File processor: "auto", "unstructured", "mistral", "kreuzberg"
    #[serde(default = "KbConfig::default_file_processor")]
    pub file_processor: String,
    /// Chunking strategy for document processing
    pub chunk_strategy: crate::uar::rag::chunking::ChunkingStrategy,
}

impl KbConfig {
    /// Default embedding provider (fastembed for local inference)
    pub fn default_embedding_provider() -> String {
        "fastembed".to_string()
    }

    /// Default embedding model
    pub fn default_embedding_model() -> String {
        "BAAI/bge-small-en-v1.5".to_string()
    }

    /// Default file processor
    pub fn default_file_processor() -> String {
        "auto".to_string()
    }
}

impl Default for KbConfig {
    fn default() -> Self {
        Self {
            embedding_provider: Self::default_embedding_provider(),
            embedding_model: Self::default_embedding_model(),
            vector_dimensions: None,
            file_processor: Self::default_file_processor(),
            chunk_strategy: crate::uar::rag::chunking::ChunkingStrategy::Recursive { size: 512 },
        }
    }
}

/// A chunk of knowledge from a processed document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: Uuid,
    pub kb_id: String,
    /// Optional reference to the source document
    #[serde(default)]
    pub document_id: Option<String>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    // Embedding is not typically serialized to frontend, but good to have
    #[serde(skip)]
    pub embedding: Vec<f32>,
    pub created_at: String, // RFC3339
}

/// A search result matching a knowledge chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMatch {
    pub chunk: KnowledgeChunk,
    pub score: f32,
}

/// Tracks a source document ingested into a knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub kb_id: String,
    pub filename: String,
    #[serde(default)]
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    #[serde(default)]
    pub chunk_count: usize,
    pub status: DocumentStatus,
    pub created_at: String, // RFC3339
    pub updated_at: String, // RFC3339
}

/// Status of document processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DocumentStatus {
    Pending,
    Processing,
    Indexed,
    Failed { error: String },
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self::Pending
    }
}
