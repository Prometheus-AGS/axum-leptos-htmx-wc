use crate::uar::domain::knowledge::KnowledgeChunk;
use crate::uar::persistence::PersistenceLayer;
use crate::uar::rag::chunking::{Chunker, ChunkingStrategy};
use crate::uar::runtime::matching::VectorMatcher;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct IngestService {
    persistence: Arc<dyn PersistenceLayer>,
    vector_matcher: Arc<VectorMatcher>,
    chunker: Chunker,
    // Track processed files to avoid re-ingesting identical content (naive check by path/mtime)
    // For MVP, we just ingest everything on startup or change.
    // Ideally store tracking info in DB.
}

impl std::fmt::Debug for IngestService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestService")
            .field("persistence", &"<dyn PersistenceLayer>")
            .field("vector_matcher", &self.vector_matcher)
            .field("chunker", &self.chunker)
            .finish()
    }
}

impl IngestService {
    pub fn new(
        persistence: Arc<dyn PersistenceLayer>,
        vector_matcher: Arc<VectorMatcher>,
        strategy: ChunkingStrategy,
    ) -> Self {
        let chunker = Chunker::new(strategy, Some(vector_matcher.clone()));
        Self {
            persistence,
            vector_matcher,
            chunker,
        }
    }

    /// Process a single file
    pub async fn ingest_file(&self, path: &Path, kb_id: &str) -> Result<()> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Only support text/md for now
        if !matches!(extension, "txt" | "md" | "markdown") {
            return Ok(()); // Skip unsupported
        }

        let content = tokio::fs::read_to_string(path).await?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        tracing::info!("Ingesting processed file: {}", filename);

        // 1. Chunking
        let chunks = self.chunker.chunk(&content).await?;

        if chunks.is_empty() {
            return Ok(());
        }

        // 2. Embedding
        let embeddings = self.vector_matcher.embed_batch(chunks.clone()).await?;

        // 3. Storage
        for (i, segment) in chunks.into_iter().enumerate() {
            let embedding = embeddings
                .get(i)
                .ok_or_else(|| anyhow!("Missing embedding for chunk {}", i))?;

            let mut metadata = HashMap::new();
            metadata.insert(
                "filename".to_string(),
                serde_json::Value::String(filename.to_string()),
            );
            metadata.insert(
                "path".to_string(),
                serde_json::Value::String(path.to_string_lossy().to_string()),
            );
            metadata.insert("index".to_string(), serde_json::json!(i));

            let chunk_id = Uuid::new_v4(); // Or deterministic based on content?

            let k_chunk = KnowledgeChunk {
                id: chunk_id,
                kb_id: kb_id.to_string(),
                document_id: None, // No document tracking in basic ingest
                content: segment,
                metadata: Some(serde_json::to_value(metadata)?),
                embedding: embedding.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            self.persistence.save_chunk(&k_chunk).await?;
        }

        Ok(())
    }

    /// Ingest text content directly (for worker pool use).
    /// Returns the number of chunks created.
    pub async fn ingest_text(
        &self,
        content: &str,
        kb_id: &str,
        document_id: String,
    ) -> Result<usize> {
        // 1. Chunking
        let chunks = self.chunker.chunk(content).await?;

        if chunks.is_empty() {
            return Ok(0);
        }

        // 2. Embedding
        let embeddings = self.vector_matcher.embed_batch(chunks.clone()).await?;

        // 3. Storage
        for (i, segment) in chunks.iter().enumerate() {
            let embedding = embeddings
                .get(i)
                .ok_or_else(|| anyhow!("Missing embedding for chunk {}", i))?;

            let mut metadata = HashMap::new();
            metadata.insert(
                "document_id".to_string(),
                serde_json::Value::String(document_id.clone()),
            );
            metadata.insert("index".to_string(), serde_json::json!(i));

            let chunk_id = Uuid::new_v4();

            let k_chunk = KnowledgeChunk {
                id: chunk_id,
                kb_id: kb_id.to_string(),
                document_id: Some(document_id.clone()),
                content: segment.clone(),
                metadata: Some(serde_json::to_value(&metadata)?),
                embedding: embedding.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            self.persistence.save_chunk(&k_chunk).await?;
        }

        Ok(chunks.len())
    }

    /// Recursively scan and ingest a directory
    pub async fn ingest_directory(&self, dir: &Path, kb_id: &str) -> Result<()> {
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Err(e) = self.ingest_file(entry.path(), kb_id).await {
                    tracing::error!("Failed to ingest {:?}: {:?}", entry.path(), e);
                }
            }
        }
        Ok(())
    }

    /// Start a watcher loop (polling)
    pub async fn watch(&self, dir: PathBuf, kb_id: String) {
        tracing::info!("Starting file watcher on {:?}", dir);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        // State to track last modified times
        let mut file_state: HashMap<PathBuf, std::time::SystemTime> = HashMap::new();

        loop {
            interval.tick().await;

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path().to_path_buf();
                    // Check modified time
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            let should_process = match file_state.get(&path) {
                                Some(last_mod) => modified > *last_mod,
                                None => true,
                            };

                            if should_process {
                                if let Err(e) = self.ingest_file(&path, &kb_id).await {
                                    tracing::error!("Watch ingest failed for {:?}: {:?}", path, e);
                                } else {
                                    file_state.insert(path, modified);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
