//! Document Ingestion Worker Pool
//!
//! Uses `prometheus_parking_lot` WorkerPool for scalable document ingestion.
//! This ensures CPU-bound document processing doesn't block the async HTTP server.

use crate::uar::{
    domain::knowledge::{DocumentStatus, KnowledgeDocument},
    persistence::PersistenceLayer,
    rag::ingest::IngestService,
};
use anyhow::Result;
use async_trait::async_trait;
use prometheus_parking_lot::{
    config::WorkerPoolConfig,
    core::{PoolError, TaskMetadata, WorkerExecutor, WorkerPool},
    util::serde::{MailboxKey, Priority, ResourceCost, ResourceKind},
};
use std::sync::Arc;
use tracing::{error, info, warn};

// =============================================================================
// Job and Result Types
// =============================================================================

/// A document ingestion job to be processed by the worker pool.
#[derive(Debug, Clone)]
pub struct DocumentIngestionJob {
    /// The document to process
    pub document: KnowledgeDocument,
    /// Raw file content (bytes)
    pub file_content: Vec<u8>,
    /// Knowledge base ID for the document
    pub kb_id: String,
}

/// Result from processing a document.
#[derive(Debug, Clone)]
pub struct IngestionResult {
    /// Document ID that was processed
    pub document_id: String,
    /// Number of chunks created
    pub chunk_count: usize,
    /// Final status
    pub status: DocumentStatus,
}

// =============================================================================
// Worker Executor Implementation
// =============================================================================

/// Executor for document ingestion jobs.
#[derive(Clone, Debug)]
pub struct DocumentIngestionExecutor {
    /// Shared ingest service for chunking and embedding
    ingest_service: Arc<IngestService>,
    /// Persistence layer for status updates
    persistence: Arc<dyn PersistenceLayer>,
}

impl DocumentIngestionExecutor {
    /// Create a new document ingestion executor.
    pub fn new(ingest_service: Arc<IngestService>, persistence: Arc<dyn PersistenceLayer>) -> Self {
        Self {
            ingest_service,
            persistence,
        }
    }
}

#[async_trait]
impl WorkerExecutor<DocumentIngestionJob, IngestionResult> for DocumentIngestionExecutor {
    async fn execute(&self, job: DocumentIngestionJob, _meta: TaskMetadata) -> IngestionResult {
        let doc_id = job.document.id.clone();
        info!(document_id = %doc_id, "Starting document ingestion");

        // Update status to Processing
        if let Err(e) = self
            .persistence
            .update_document_status(&doc_id, &DocumentStatus::Processing)
            .await
        {
            warn!(document_id = %doc_id, error = %e, "Failed to update status to processing");
        }

        // Attempt to ingest the document
        match self.process_document(&job).await {
            Ok(chunk_count) => {
                // Update status to Indexed
                let status = DocumentStatus::Indexed;
                if let Err(e) = self
                    .persistence
                    .update_document_status(&doc_id, &status)
                    .await
                {
                    error!(document_id = %doc_id, error = %e, "Failed to update status to indexed");
                }

                info!(document_id = %doc_id, chunk_count, "Document ingestion completed");
                IngestionResult {
                    document_id: doc_id,
                    chunk_count,
                    status,
                }
            }
            Err(e) => {
                // Update status to Failed
                let status = DocumentStatus::Failed {
                    error: e.to_string(),
                };
                if let Err(update_err) = self
                    .persistence
                    .update_document_status(&doc_id, &status)
                    .await
                {
                    error!(document_id = %doc_id, error = %update_err, "Failed to update status to failed");
                }

                error!(document_id = %doc_id, error = %e, "Document ingestion failed");
                IngestionResult {
                    document_id: doc_id,
                    chunk_count: 0,
                    status,
                }
            }
        }
    }
}

impl DocumentIngestionExecutor {
    /// Process a document and return chunk count.
    async fn process_document(&self, job: &DocumentIngestionJob) -> Result<usize> {
        // Convert file content to text (for now, assume text files)
        // In production, this would use file processors (Kreuzberg, etc.)
        let text = String::from_utf8_lossy(&job.file_content);

        // Use the ingest service to chunk, embed, and store
        let chunks = self
            .ingest_service
            .ingest_text(&text, &job.kb_id, job.document.id.clone())
            .await?;

        Ok(chunks)
    }
}

// =============================================================================
// Worker Pool Wrapper
// =============================================================================

/// High-level wrapper around the `prometheus_parking_lot` WorkerPool
/// for document ingestion.
pub struct IngestionWorkerPool {
    /// The underlying worker pool
    pool: WorkerPool<DocumentIngestionJob, IngestionResult, DocumentIngestionExecutor>,
}

impl std::fmt::Debug for IngestionWorkerPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestionWorkerPool").finish()
    }
}

impl IngestionWorkerPool {
    /// Create a new ingestion worker pool.
    ///
    /// # Arguments
    /// * `worker_count` - Number of worker threads (0 = auto-detect based on CPU)
    /// * `max_queue_depth` - Maximum pending jobs before backpressure
    /// * `ingest_service` - Shared ingest service
    /// * `persistence` - Persistence layer for status updates
    pub fn new(
        worker_count: usize,
        max_queue_depth: usize,
        ingest_service: Arc<IngestService>,
        persistence: Arc<dyn PersistenceLayer>,
    ) -> Result<Self, PoolError> {
        let worker_count = if worker_count == 0 {
            num_cpus::get()
        } else {
            worker_count
        };

        let config = WorkerPoolConfig::new()
            .with_worker_count(worker_count)
            .with_max_units(1000) // Resource capacity
            .with_max_queue_depth(max_queue_depth);

        let executor = DocumentIngestionExecutor::new(ingest_service, persistence);
        let pool = WorkerPool::new(config, executor)?;

        info!(
            worker_count,
            max_queue_depth, "Ingestion worker pool initialized"
        );

        Ok(Self { pool })
    }

    /// Submit a document for ingestion.
    ///
    /// Returns a job key that can be used to retrieve the result.
    pub async fn submit(
        &self,
        document: KnowledgeDocument,
        file_content: Vec<u8>,
    ) -> Result<String, PoolError> {
        let job = DocumentIngestionJob {
            kb_id: document.kb_id.clone(),
            document,
            file_content,
        };

        let meta = TaskMetadata {
            id: uuid::Uuid::new_v4().as_u128() as u64,
            priority: Priority::Normal,
            cost: ResourceCost {
                kind: ResourceKind::Cpu, // CPU-bound work
                units: 10,               // Each document uses 10 resource units
            },
            created_at_ms: chrono::Utc::now().timestamp_millis() as u128,
            deadline_ms: None,
            mailbox: None,
        };

        let key = self.pool.submit_async(job, meta).await?;
        Ok(format!("{key:?}"))
    }

    /// Retrieve the result of an ingestion job.
    ///
    /// Blocks until the job completes or the timeout expires.
    pub async fn retrieve(
        &self,
        key: &MailboxKey,
        timeout: std::time::Duration,
    ) -> Result<IngestionResult, PoolError> {
        self.pool.retrieve_async(key, timeout).await
    }

    /// Shutdown the worker pool gracefully.
    pub fn shutdown(self) {
        drop(self.pool);
    }
}
