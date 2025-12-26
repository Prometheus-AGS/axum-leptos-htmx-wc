//! GraphRAG Domain Models
//!
//! Entity, relationship, community, and citation structures for
//! knowledge graph-enhanced retrieval.

use serde::{Deserialize, Serialize};

// =============================================================================
// Entity Types
// =============================================================================

/// Types of entities that can be extracted from documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// A person or individual
    Person,
    /// An organization, company, or institution
    Organization,
    /// A geographical or political location
    Location,
    /// An event or occurrence
    Event,
    /// A technical or abstract concept
    Concept,
    /// A product, technology, or artifact
    Product,
    /// Date or time reference
    Temporal,
    /// Numeric or quantitative data
    Quantity,
    /// Custom entity type
    Custom(String),
}

impl Default for EntityType {
    fn default() -> Self {
        Self::Concept
    }
}

// =============================================================================
// Entity
// =============================================================================

/// An entity extracted from document content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier
    pub id: String,
    /// Canonical (normalized) name
    pub canonical_name: String,
    /// Entity type classification
    pub entity_type: EntityType,
    /// LLM-generated description of the entity
    pub description: Option<String>,
    /// Vector embedding for similarity search
    pub embedding: Vec<f32>,
    /// IDs of source chunks this entity was extracted from
    pub source_chunk_ids: Vec<String>,
    /// Creation timestamp
    pub created_at: String,
}

// =============================================================================
// Relationship
// =============================================================================

/// A directed relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier
    pub id: String,
    /// Source entity ID
    pub source_id: String,
    /// Target entity ID
    pub target_id: String,
    /// Relationship type (e.g., "works_at", "mentions", "depends_on")
    pub relation_type: String,
    /// Confidence/weight of the relationship (0.0 - 1.0)
    pub weight: f32,
    /// LLM-generated description of the relationship
    pub description: Option<String>,
    /// ID of the source chunk where this relationship was found
    pub source_chunk_id: String,
    /// Creation timestamp
    pub created_at: String,
}

// =============================================================================
// Community
// =============================================================================

/// A community (cluster) of related entities detected via graph algorithms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    /// Unique identifier
    pub id: String,
    /// Hierarchy level (0 = leaf, higher = more abstract)
    pub level: u32,
    /// IDs of entities in this community
    pub entity_ids: Vec<String>,
    /// LLM-generated summary of the community
    pub summary: String,
    /// Vector embedding of the summary for search
    pub embedding: Vec<f32>,
    /// Creation timestamp
    pub created_at: String,
}

// =============================================================================
// Citation
// =============================================================================

/// A citation referencing a source chunk used in RAG response generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Chunk ID referenced
    pub chunk_id: String,
    /// Document ID (if known)
    pub document_id: Option<String>,
    /// Human-readable document name
    pub document_name: String,
    /// Relevance score for this citation
    pub relevance_score: f32,
    /// Snippet of the relevant content
    pub snippet: String,
    /// Entities mentioned in this chunk
    pub entities_mentioned: Vec<String>,
}

// =============================================================================
// RAG Context
// =============================================================================

/// Full context assembled for RAG query processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    /// Retrieved chunks
    pub chunks: Vec<crate::uar::domain::knowledge::KnowledgeChunk>,
    /// Citations for attribution
    pub citations: Vec<Citation>,
    /// Relevant entities
    pub entities: Vec<Entity>,
    /// Relationships between entities
    pub relationships: Vec<Relationship>,
    /// Community summaries for global context
    pub community_summaries: Vec<String>,
}

impl Default for RAGContext {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            citations: Vec::new(),
            entities: Vec::new(),
            relationships: Vec::new(),
            community_summaries: Vec::new(),
        }
    }
}

// =============================================================================
// Extraction Result
// =============================================================================

/// Result of entity/relationship extraction from a chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Extracted entities
    pub entities: Vec<Entity>,
    /// Extracted relationships
    pub relationships: Vec<Relationship>,
}

impl Default for ExtractionResult {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            relationships: Vec::new(),
        }
    }
}
