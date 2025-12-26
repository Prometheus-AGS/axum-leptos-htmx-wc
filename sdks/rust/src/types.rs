//! Shared types for the SDK.
//!
//! These types mirror the server's API DTOs and are used for both
//! HTTP client and embedded runtime modes.

use serde::{Deserialize, Serialize};

// =============================================================================
// Chat API Types
// =============================================================================

/// Request to start a chat session.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// The user's message.
    pub message: String,
    /// Optional session ID to continue an existing conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Response from starting a chat.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// The session ID for this conversation.
    pub session_id: String,
    /// URL to stream chat events from.
    pub stream_url: String,
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: String,
    /// The message content.
    pub content: String,
}

// =============================================================================
// Runs API Types
// =============================================================================

/// Request to create a new agent run.
#[derive(Debug, Clone, Serialize)]
pub struct CreateRunRequest {
    /// The input prompt for the run.
    pub input: String,
    /// Optional context or configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// Response from creating a run.
#[derive(Debug, Clone, Deserialize)]
pub struct RunResponse {
    /// The unique run ID.
    pub id: String,
    /// URL to stream run events from.
    pub stream_url: String,
}

// =============================================================================
// Knowledge Base API Types
// =============================================================================

/// A knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Configuration settings.
    #[serde(default)]
    pub config: KnowledgeBaseConfig,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Knowledge base configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    /// Embedding provider name.
    #[serde(default)]
    pub embedding_provider: String,
    /// Embedding model name.
    #[serde(default)]
    pub embedding_model: String,
    /// Vector dimensions (if known).
    pub vector_dimensions: Option<usize>,
    /// File processor to use.
    #[serde(default)]
    pub file_processor: String,
    /// Chunking strategy.
    #[serde(default)]
    pub chunk_strategy: String,
}

/// Request to create a knowledge base.
#[derive(Debug, Clone, Serialize)]
pub struct CreateKnowledgeBaseRequest {
    /// Name of the knowledge base.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A document in a knowledge base.
#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    /// Unique identifier.
    pub id: String,
    /// Knowledge base ID.
    pub kb_id: String,
    /// Original filename.
    pub filename: String,
    /// MIME type.
    pub mime_type: Option<String>,
    /// Number of chunks created.
    pub chunk_count: usize,
    /// Processing status.
    pub status: String,
    /// Error message if failed.
    pub error_message: Option<String>,
}

/// Request to search a knowledge base.
#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    /// The search query.
    pub query: String,
    /// Maximum number of results.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Minimum similarity score.
    #[serde(default = "default_min_score")]
    pub min_score: f32,
}

fn default_limit() -> usize {
    5
}
fn default_min_score() -> f32 {
    0.7
}

/// Response from a knowledge base search.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    /// The search results.
    pub results: Vec<SearchResult>,
}

/// A single search result.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    /// The content of the matching chunk.
    pub content: String,
    /// Similarity score.
    pub score: f32,
    /// Additional metadata.
    pub metadata: serde_json::Value,
    /// Optional document ID.
    pub document_id: Option<String>,
}

// =============================================================================
// Ingest API Types
// =============================================================================

/// Request to ingest content.
#[derive(Debug, Clone, Serialize)]
pub struct IngestRequest {
    /// The content to ingest.
    pub content: String,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Response from ingestion.
#[derive(Debug, Clone, Deserialize)]
pub struct IngestResponse {
    /// Whether ingestion was successful.
    pub success: bool,
    /// Number of chunks created.
    pub chunk_count: usize,
}
