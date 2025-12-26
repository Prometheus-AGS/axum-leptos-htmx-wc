//! Hybrid Retriever with Reciprocal Rank Fusion (RRF)
//!
//! Combines vector search results with graph traversal results
//! using RRF for optimal ranking.

use crate::uar::domain::{
    graph::{Citation, RAGContext},
    knowledge::{KnowledgeChunk, KnowledgeMatch},
};
use std::collections::HashMap;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for hybrid retrieval.
#[derive(Debug, Clone)]
pub struct HybridRetrieverConfig {
    /// Weight for vector search results (0.0 to 1.0)
    pub vector_weight: f32,
    /// Weight for graph results (0.0 to 1.0)
    pub graph_weight: f32,
    /// RRF constant k (typically 60)
    pub rrf_k: f32,
    /// Maximum results to return
    pub max_results: usize,
    /// Minimum score threshold
    pub min_score: f32,
}

impl Default for HybridRetrieverConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            graph_weight: 0.3,
            rrf_k: 60.0,
            max_results: 10,
            min_score: 0.0,
        }
    }
}

// =============================================================================
// Retrieval Source
// =============================================================================

/// Source of a retrieved chunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetrievalSource {
    Vector,
    Graph,
    Both,
}

/// A scored chunk with source attribution.
#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub chunk: KnowledgeChunk,
    pub score: f32,
    pub vector_rank: Option<usize>,
    pub graph_rank: Option<usize>,
    pub source: RetrievalSource,
}

// =============================================================================
// Hybrid Retriever
// =============================================================================

/// Combines vector and graph retrieval with RRF.
#[derive(Debug, Clone)]
pub struct HybridRetriever {
    config: HybridRetrieverConfig,
}

impl HybridRetriever {
    /// Create a new hybrid retriever with default config.
    pub fn new() -> Self {
        Self {
            config: HybridRetrieverConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: HybridRetrieverConfig) -> Self {
        Self { config }
    }

    /// Combine vector search results with graph results using RRF.
    ///
    /// # Arguments
    /// * `vector_results` - Results from vector similarity search
    /// * `graph_results` - Results from graph traversal
    ///
    /// # Returns
    /// Combined and re-ranked results
    pub fn fuse(
        &self,
        vector_results: Vec<KnowledgeMatch>,
        graph_results: Vec<KnowledgeMatch>,
    ) -> Vec<ScoredChunk> {
        let mut chunk_scores: HashMap<String, ScoredChunk> = HashMap::new();

        // Process vector results
        for (rank, m) in vector_results.into_iter().enumerate() {
            let chunk_id = m.chunk.id.to_string();
            let rrf_score = self.rrf_score(rank, self.config.vector_weight);

            chunk_scores.insert(
                chunk_id,
                ScoredChunk {
                    chunk: m.chunk,
                    score: rrf_score,
                    vector_rank: Some(rank),
                    graph_rank: None,
                    source: RetrievalSource::Vector,
                },
            );
        }

        // Process graph results and combine
        for (rank, m) in graph_results.into_iter().enumerate() {
            let chunk_id = m.chunk.id.to_string();
            let rrf_score = self.rrf_score(rank, self.config.graph_weight);

            if let Some(existing) = chunk_scores.get_mut(&chunk_id) {
                // Found in both sources
                existing.score += rrf_score;
                existing.graph_rank = Some(rank);
                existing.source = RetrievalSource::Both;
            } else {
                chunk_scores.insert(
                    chunk_id,
                    ScoredChunk {
                        chunk: m.chunk,
                        score: rrf_score,
                        vector_rank: None,
                        graph_rank: Some(rank),
                        source: RetrievalSource::Graph,
                    },
                );
            }
        }

        // Sort by combined score
        let mut results: Vec<ScoredChunk> = chunk_scores
            .into_values()
            .filter(|c| c.score >= self.config.min_score)
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(self.config.max_results);

        tracing::debug!("RRF fusion: {} chunks from vector + graph", results.len());

        results
    }

    /// Calculate RRF score for a result at a given rank.
    fn rrf_score(&self, rank: usize, weight: f32) -> f32 {
        weight / (self.config.rrf_k + rank as f32 + 1.0)
    }

    /// Build RAG context from fused results with citations.
    pub fn build_context(
        &self,
        results: Vec<ScoredChunk>,
        document_names: &HashMap<String, String>,
    ) -> RAGContext {
        let mut context = RAGContext::default();

        for scored in results {
            // Add chunk
            context.chunks.push(scored.chunk.clone());

            // Build citation
            let doc_name = scored
                .chunk
                .document_id
                .as_ref()
                .and_then(|did| document_names.get(did))
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            let citation = Citation {
                chunk_id: scored.chunk.id.to_string(),
                document_id: scored.chunk.document_id.clone(),
                document_name: doc_name,
                relevance_score: scored.score,
                snippet: scored.chunk.content.chars().take(200).collect(),
                entities_mentioned: Vec::new(), // Would be populated by entity extractor
            };

            context.citations.push(citation);
        }

        context
    }
}

impl Default for HybridRetriever {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_chunk(id: &str, content: &str) -> KnowledgeChunk {
        KnowledgeChunk {
            id: Uuid::new_v4(),
            kb_id: "test-kb".to_string(),
            document_id: Some(id.to_string()),
            content: content.to_string(),
            metadata: None,
            embedding: Vec::new(),
            created_at: "2024-01-01".to_string(),
        }
    }

    #[test]
    fn test_rrf_fusion_empty() {
        let retriever = HybridRetriever::new();
        let results = retriever.fuse(vec![], vec![]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_fusion_vector_only() {
        let retriever = HybridRetriever::new();
        let vector_results = vec![KnowledgeMatch {
            chunk: make_chunk("doc1", "Test content"),
            score: 0.9,
        }];

        let results = retriever.fuse(vector_results, vec![]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, RetrievalSource::Vector);
    }

    #[test]
    fn test_rrf_fusion_both_sources() {
        let retriever = HybridRetriever::new();
        let chunk = make_chunk("doc1", "Test content");

        let vector_results = vec![KnowledgeMatch {
            chunk: chunk.clone(),
            score: 0.9,
        }];
        let graph_results = vec![KnowledgeMatch {
            chunk: chunk.clone(),
            score: 0.8,
        }];

        let results = retriever.fuse(vector_results, graph_results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, RetrievalSource::Both);
        // Score should be sum of both RRF scores
        assert!(results[0].score > 0.0);
    }
}
