//! Entity and Relationship Extraction Strategies
//!
//! Provides the trait interface and implementations for extracting
//! entities and relationships from document chunks.

pub mod external_nlp;
pub mod leiden;

use crate::uar::domain::{graph::ExtractionResult, knowledge::KnowledgeChunk};
use anyhow::Result;
use async_trait::async_trait;

// =============================================================================
// Extraction Strategy Trait
// =============================================================================

/// Strategy for extracting entities and relationships from text.
#[async_trait]
pub trait RelationshipExtractor: Send + Sync {
    /// Extract entities and relationships from a knowledge chunk.
    async fn extract(&self, chunk: &KnowledgeChunk) -> Result<ExtractionResult>;

    /// Extract from raw text (convenience method).
    async fn extract_from_text(&self, text: &str) -> Result<ExtractionResult>;

    /// Get the name of this extraction strategy.
    fn name(&self) -> &'static str;
}

// =============================================================================
// Extraction Configuration
// =============================================================================

/// Configuration for extraction strategies.
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Whether to extract implicit relationships
    pub extract_implicit: bool,
    /// Minimum confidence for entity extraction
    pub min_confidence: f32,
    /// Maximum entities per chunk
    pub max_entities: usize,
    /// Entity types to extract (empty = all)
    pub entity_types: Vec<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            extract_implicit: true,
            min_confidence: 0.5,
            max_entities: 50,
            entity_types: Vec::new(),
        }
    }
}
