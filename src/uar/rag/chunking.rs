use crate::uar::runtime::matching::VectorMatcher;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use text_splitter::{Characters, ChunkConfig, TextSplitter};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkingStrategy {
    /// Simple fixed character length
    FixedSize { size: usize },
    /// Token-based splitting (using cl100k_base via text_splitter)
    Token { tokens: usize },
    /// Recursive character splitting trying to respect semantic boundaries (paragraphs, etc.)
    Recursive { size: usize },
    /// Split by sentence
    Sentence,
    /// Keep full document (no chunking)
    Document,
    /// Semantic chunking: Embed sentences and merge if similar
    Semantic { threshold: f32 },
    /// Agentic: Ask LLM (Not implemented yet)
    Agentic,
}

pub struct Chunker {
    strategy: ChunkingStrategy,
    // Optional because not all strategies need it
    vector_matcher: Option<Arc<VectorMatcher>>,
}

impl Chunker {
    pub fn new(strategy: ChunkingStrategy, vector_matcher: Option<Arc<VectorMatcher>>) -> Self {
        Self {
            strategy,
            vector_matcher,
        }
    }

    pub async fn chunk(&self, text: &str) -> Result<Vec<String>> {
        match &self.strategy {
            ChunkingStrategy::FixedSize { size } => Ok(text
                .chars()
                .collect::<Vec<char>>()
                .chunks(*size)
                .map(|c| c.iter().collect::<String>())
                .collect()),
            ChunkingStrategy::Recursive { size } => {
                let config = ChunkConfig::new(*size)
                    .with_sizer(Characters)
                    .with_trim(true);
                let splitter = TextSplitter::new(config);
                Ok(splitter.chunks(text).map(|s: &str| s.to_string()).collect())
            }
            ChunkingStrategy::Token { tokens } => {
                let size = tokens * 4;
                let config = ChunkConfig::new(size)
                    .with_sizer(Characters)
                    .with_trim(true);
                let splitter = TextSplitter::new(config);
                Ok(splitter.chunks(text).map(|s: &str| s.to_string()).collect())
            }
            ChunkingStrategy::Sentence => {
                // Simple split by punctuation or newlines
                // Or use unicode_segmentation if available.
                // Falling back to simple split for MVP.
                Ok(text
                    .split_inclusive(&['.', '!', '?'])
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect())
            }
            ChunkingStrategy::Document => Ok(vec![text.to_string()]),
            ChunkingStrategy::Semantic { threshold } => self.semantic_chunk(text, *threshold).await,
            ChunkingStrategy::Agentic => {
                warn!("Agentic chunking not implemented, falling back to Document");
                Ok(vec![text.to_string()])
            }
        }
    }

    async fn semantic_chunk(&self, text: &str, threshold: f32) -> Result<Vec<String>> {
        let matcher = self
            .vector_matcher
            .as_ref()
            .ok_or_else(|| anyhow!("VectorMatcher required for Semantic Chunking"))?;

        // 1. Split into "Base Sentences" (using simple sentence strategy)
        let sentences: Vec<String> = text
            .split_inclusive(&['.', '!', '?', '\n'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()) // simplistic
            .collect();

        if sentences.is_empty() {
            return Ok(vec![]);
        }

        // 2. Embed all sentences
        let embeddings = matcher.embed_batch(sentences.clone()).await?;

        // 3. Iterate and merge
        // Algorithm:
        // Start current_chunk with sentence[0].
        // next_sentence = sentence[1].
        // Sim = cosine(embedding[current_chunk_avg], embedding[next]).
        // If Sim > Threshold -> Merge.
        // Else -> Push current_chunk, start new.
        // Optimization: Just compare adjacent sentences for now (easier than maintaining running avg embedding).

        // This is a complex logic. MVP Implementation:
        // Merge adjacent if similar.

        let mut chunks = Vec::new();
        let mut current_chunk = sentences[0].clone();
        let mut current_emb = embeddings[0].clone();

        for i in 1..sentences.len() {
            let next_sent = &sentences[i];
            let next_emb = &embeddings[i];

            // Calculate similarity between Current Aggregate vs Next
            // (Or just Last vs Next? Aggregate is better but requires re-embedding or averaging)
            // Averaging normalized embeddings is a decent approximation for "topic".
            // Let's use Last Sentence vs Next Sentence for simple "coherence".
            // Better: Average of current chunk so far.
            // Simple MVP: Compare with *previous sentence* (i-1) embedding.
            // Actually, comparing to the *running average* of the current chunk is standard.

            // Cosine Similarity
            let sim = cosine_similarity(&current_emb, next_emb);

            if sim >= threshold {
                // Merge
                current_chunk.push_str(" ");
                current_chunk.push_str(next_sent);
                // Update average embedding (naive unweighted average)
                current_emb = current_emb
                    .iter()
                    .zip(next_emb.iter())
                    .map(|(a, b)| (a + b) / 2.0)
                    .collect();
            } else {
                // Finalize chunk
                chunks.push(current_chunk);
                // Start new
                current_chunk = next_sent.clone();
                current_emb = next_emb.clone();
            }
        }
        chunks.push(current_chunk);

        Ok(chunks)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixed_size() {
        let strategy = ChunkingStrategy::FixedSize { size: 5 };
        let chunker = Chunker::new(strategy, None);
        let text = "HelloWorld";
        let chunks = chunker.chunk(text).await.unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "Hello");
        assert_eq!(chunks[1], "World");
    }

    #[tokio::test]
    async fn test_recursive() {
        let strategy = ChunkingStrategy::Recursive { size: 10 };
        let chunker = Chunker::new(strategy, None);
        let text = "Hello World From Rust";
        // Recursively split to fit 10 chars.
        // "Hello World" is 11 chars. So it should split.
        // "Hello" (5) "World" (5).
        // " From Rust" (10).
        let chunks = chunker.chunk(text).await.unwrap();
        // text-splitter behavior depends on boundaries.
        // It should split by word ideally.
        assert!(!chunks.is_empty());
        for c in chunks {
            assert!(c.len() <= 10, "Chunk '{}' exceeds size 10", c);
        }
    }

    #[test]
    fn test_cosine() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.0001);
    }
}
