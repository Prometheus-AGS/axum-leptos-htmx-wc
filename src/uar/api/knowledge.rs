//! REST API routes for Knowledge Base and Document management.
//!
//! Provides CRUD operations for knowledge bases and document ingestion.

use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::uar::{
    domain::knowledge::{DocumentStatus, KbConfig, KnowledgeBase, KnowledgeDocument},
    persistence::PersistenceLayer,
    rag::{chunking::ChunkingStrategy, ingestion_worker::IngestionWorkerPool},
    runtime::matching::VectorMatcher,
};

// =============================================================================
// State Wrapper (shares persistence layer)
// =============================================================================

/// Shared state for knowledge base API handlers.
#[derive(Clone, Debug)]
pub struct KnowledgeApiState {
    pub persistence: Arc<dyn PersistenceLayer>,
    pub vector_matcher: Arc<VectorMatcher>,
    pub ingestion_pool: Option<Arc<IngestionWorkerPool>>,
}

// =============================================================================
// Request/Response DTOs
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub config: Option<KbConfigRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKnowledgeBaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<KbConfigRequest>,
}

#[derive(Debug, Deserialize)]
pub struct KbConfigRequest {
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
    pub vector_dimensions: Option<usize>,
    pub file_processor: Option<String>,
    pub chunk_strategy: Option<String>,
    pub chunk_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeBaseResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: KbConfigResponse,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct KbConfigResponse {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub vector_dimensions: Option<usize>,
    pub file_processor: String,
    pub chunk_strategy: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub kb_id: String,
    pub filename: String,
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    pub chunk_count: usize,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_min_score")]
    pub min_score: f32,
}

fn default_limit() -> usize {
    5
}
fn default_min_score() -> f32 {
    0.7
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
    pub document_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_page_limit")]
    pub limit: usize,
}

fn default_page_limit() -> usize {
    50
}

// =============================================================================
// Router Builder
// =============================================================================

/// Build the knowledge base API router.
pub fn build_router() -> Router<Arc<KnowledgeApiState>> {
    Router::new()
        // Knowledge Base CRUD
        .route("/", get(list_knowledge_bases).post(create_knowledge_base))
        .route(
            "/{id}",
            get(get_knowledge_base)
                .put(update_knowledge_base)
                .delete(delete_knowledge_base),
        )
        // Documents
        .route("/{id}/documents", get(list_documents).post(upload_document))
        .route(
            "/{id}/documents/{doc_id}",
            get(get_document).delete(delete_document),
        )
        // Search
        .route("/{id}/search", post(search_knowledge_base))
}

// =============================================================================
// Knowledge Base Handlers
// =============================================================================

/// GET / - List all knowledge bases
async fn list_knowledge_bases(
    State(state): State<Arc<KnowledgeApiState>>,
) -> Result<Json<Vec<KnowledgeBaseResponse>>, (StatusCode, String)> {
    let kbs = state
        .persistence
        .list_knowledge_bases()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<KnowledgeBaseResponse> = kbs.into_iter().map(kb_to_response).collect();
    Ok(Json(responses))
}

/// POST / - Create a new knowledge base
async fn create_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Json(req): Json<CreateKnowledgeBaseRequest>,
) -> Result<(StatusCode, Json<KnowledgeBaseResponse>), (StatusCode, String)> {
    // Check if name already exists
    if let Ok(Some(_)) = state
        .persistence
        .get_knowledge_base_by_name(&req.name)
        .await
    {
        return Err((
            StatusCode::CONFLICT,
            format!("Knowledge base with name '{}' already exists", req.name),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let config = build_kb_config(req.config);

    let kb = KnowledgeBase {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        description: req.description,
        config,
        created_at: now.clone(),
        updated_at: now,
    };

    state
        .persistence
        .save_knowledge_base(&kb)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("Created knowledge base: {} ({})", kb.name, kb.id);
    Ok((StatusCode::CREATED, Json(kb_to_response(kb))))
}

/// GET /{id} - Get a knowledge base by ID
async fn get_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Path(id): Path<String>,
) -> Result<Json<KnowledgeBaseResponse>, (StatusCode, String)> {
    let kb = state
        .persistence
        .get_knowledge_base(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{}' not found", id),
        ))?;

    Ok(Json(kb_to_response(kb)))
}

/// PUT /{id} - Update a knowledge base
async fn update_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateKnowledgeBaseRequest>,
) -> Result<Json<KnowledgeBaseResponse>, (StatusCode, String)> {
    let mut kb = state
        .persistence
        .get_knowledge_base(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{}' not found", id),
        ))?;

    // Apply updates
    if let Some(name) = req.name {
        // Check uniqueness
        if let Ok(Some(existing)) = state.persistence.get_knowledge_base_by_name(&name).await {
            if existing.id != kb.id {
                return Err((
                    StatusCode::CONFLICT,
                    format!("Knowledge base with name '{}' already exists", name),
                ));
            }
        }
        kb.name = name;
    }
    if let Some(desc) = req.description {
        kb.description = Some(desc);
    }
    if let Some(cfg_req) = req.config {
        kb.config = merge_kb_config(kb.config, cfg_req);
    }
    kb.updated_at = chrono::Utc::now().to_rfc3339();

    state
        .persistence
        .save_knowledge_base(&kb)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(kb_to_response(kb)))
}

/// DELETE /{id} - Delete a knowledge base
async fn delete_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Check exists
    let _ = state
        .persistence
        .get_knowledge_base(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{}' not found", id),
        ))?;

    state
        .persistence
        .delete_knowledge_base(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("Deleted knowledge base: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Document Handlers
// =============================================================================

/// GET /{id}/documents - List documents in a knowledge base
async fn list_documents(
    State(state): State<Arc<KnowledgeApiState>>,
    Path(kb_id): Path<String>,
    Query(_query): Query<ListQuery>,
) -> Result<Json<Vec<DocumentResponse>>, (StatusCode, String)> {
    // Verify KB exists
    let _ = state
        .persistence
        .get_knowledge_base(&kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{}' not found", kb_id),
        ))?;

    let docs = state
        .persistence
        .list_documents(&kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<DocumentResponse> = docs.into_iter().map(doc_to_response).collect();
    Ok(Json(responses))
}

/// POST /{id}/documents - Upload a document (multipart form)
async fn upload_document(
    State(state): State<Arc<KnowledgeApiState>>,
    Path(kb_id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DocumentResponse>), (StatusCode, String)> {
    // Verify KB exists
    let _ = state
        .persistence
        .get_knowledge_base(&kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{}' not found", kb_id),
        ))?;

    // Read multipart file
    let mut filename = String::new();
    let mut mime_type = None;
    let mut file_data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            filename = field.file_name().unwrap_or("uploaded_file").to_string();
            mime_type = field.content_type().map(|s| s.to_string());
            file_data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                .to_vec();
        }
    }

    if filename.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No file field in multipart form".to_string(),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let doc = KnowledgeDocument {
        id: uuid::Uuid::new_v4().to_string(),
        kb_id: kb_id.clone(),
        filename: filename.clone(),
        file_path: None, // Would be set after saving to storage
        mime_type,
        chunk_count: 0,
        status: DocumentStatus::Pending,
        created_at: now.clone(),
        updated_at: now,
    };

    state
        .persistence
        .save_document(&doc)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Submit to worker pool for async processing
    if let Some(pool) = &state.ingestion_pool {
        match pool.submit(doc.clone(), file_data).await {
            Ok(job_key) => {
                tracing::info!(
                    document_id = %doc.id,
                    job_key = %job_key,
                    "Document submitted to ingestion queue"
                );
            }
            Err(e) => {
                tracing::error!(
                    document_id = %doc.id,
                    error = %e,
                    "Failed to submit to ingestion queue"
                );
                // Still return success - document is saved, will be processed later
            }
        }
    } else {
        tracing::warn!(
            document_id = %doc.id,
            "No ingestion pool configured - document saved but not processed"
        );
    }

    tracing::info!("Document uploaded: {} -> KB {}", doc.id, kb_id);
    Ok((StatusCode::ACCEPTED, Json(doc_to_response(doc))))
}

/// GET /{id}/documents/{doc_id} - Get document status
async fn get_document(
    State(state): State<Arc<KnowledgeApiState>>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<Json<DocumentResponse>, (StatusCode, String)> {
    let doc = state
        .persistence
        .get_document(&doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Document '{}' not found", doc_id),
        ))?;

    // Verify document belongs to the KB
    if doc.kb_id != kb_id {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Document '{}' not found in KB '{}'", doc_id, kb_id),
        ));
    }

    Ok(Json(doc_to_response(doc)))
}

/// DELETE /{id}/documents/{doc_id} - Delete a document
async fn delete_document(
    State(state): State<Arc<KnowledgeApiState>>,
    Path((kb_id, doc_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Verify document exists and belongs to KB
    let doc = state
        .persistence
        .get_document(&doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Document '{}' not found", doc_id),
        ))?;

    if doc.kb_id != kb_id {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Document '{}' not found in KB '{}'", doc_id, kb_id),
        ));
    }

    // Delete document and its chunks
    state
        .persistence
        .delete_document(&doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("Deleted document: {} from KB {}", doc_id, kb_id);
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Search Handler
// =============================================================================

/// POST /{id}/search - Vector search within a knowledge base
async fn search_knowledge_base(
    State(state): State<Arc<KnowledgeApiState>>,
    Path(kb_id): Path<String>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    // Verify KB exists
    let kb = state
        .persistence
        .get_knowledge_base(&kb_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("Knowledge base '{}' not found", kb_id),
        ))?;

    tracing::debug!(
        "Search in KB '{}': query='{}', limit={}",
        kb.name,
        req.query,
        req.limit
    );

    // Embed the query using VectorMatcher
    let embeddings = state
        .vector_matcher
        .embed_batch(vec![req.query.clone()])
        .await
        .map_err(|e| {
            tracing::error!("Failed to embed query: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Embedding failed: {}", e),
            )
        })?;

    let query_vec = embeddings.into_iter().next().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No embedding generated".to_string(),
    ))?;

    // Search knowledge scoped to this KB
    let matches = state
        .persistence
        .search_knowledge_scoped(&[kb_id.as_str()], &query_vec, req.limit, req.min_score)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Transform to response
    let results = matches
        .into_iter()
        .map(|m| SearchResult {
            content: m.chunk.content,
            score: m.score,
            metadata: m.chunk.metadata.unwrap_or_else(|| serde_json::json!({})),
            document_id: m.chunk.document_id,
        })
        .collect();

    Ok(Json(SearchResponse { results }))
}

// =============================================================================
// Helper Functions
// =============================================================================

fn kb_to_response(kb: KnowledgeBase) -> KnowledgeBaseResponse {
    KnowledgeBaseResponse {
        id: kb.id,
        name: kb.name,
        description: kb.description,
        config: KbConfigResponse {
            embedding_provider: kb.config.embedding_provider,
            embedding_model: kb.config.embedding_model,
            vector_dimensions: kb.config.vector_dimensions,
            file_processor: kb.config.file_processor,
            chunk_strategy: format!("{:?}", kb.config.chunk_strategy),
        },
        created_at: kb.created_at,
        updated_at: kb.updated_at,
    }
}

fn doc_to_response(doc: KnowledgeDocument) -> DocumentResponse {
    let (status_str, error_msg) = match &doc.status {
        DocumentStatus::Pending => ("pending".to_string(), None),
        DocumentStatus::Processing => ("processing".to_string(), None),
        DocumentStatus::Indexed => ("indexed".to_string(), None),
        DocumentStatus::Failed { error } => ("failed".to_string(), Some(error.clone())),
    };

    DocumentResponse {
        id: doc.id,
        kb_id: doc.kb_id,
        filename: doc.filename,
        file_path: doc.file_path,
        mime_type: doc.mime_type,
        chunk_count: doc.chunk_count,
        status: status_str,
        error_message: error_msg,
        created_at: doc.created_at,
        updated_at: doc.updated_at,
    }
}

fn build_kb_config(req: Option<KbConfigRequest>) -> KbConfig {
    match req {
        Some(cfg) => KbConfig {
            embedding_provider: cfg
                .embedding_provider
                .unwrap_or_else(KbConfig::default_embedding_provider),
            embedding_model: cfg
                .embedding_model
                .unwrap_or_else(KbConfig::default_embedding_model),
            vector_dimensions: cfg.vector_dimensions,
            file_processor: cfg
                .file_processor
                .unwrap_or_else(KbConfig::default_file_processor),
            chunk_strategy: parse_chunk_strategy(cfg.chunk_strategy.as_deref(), cfg.chunk_size),
        },
        None => KbConfig::default(),
    }
}

fn merge_kb_config(mut existing: KbConfig, req: KbConfigRequest) -> KbConfig {
    if let Some(provider) = req.embedding_provider {
        existing.embedding_provider = provider;
    }
    if let Some(model) = req.embedding_model {
        existing.embedding_model = model;
    }
    if req.vector_dimensions.is_some() {
        existing.vector_dimensions = req.vector_dimensions;
    }
    if let Some(processor) = req.file_processor {
        existing.file_processor = processor;
    }
    if req.chunk_strategy.is_some() || req.chunk_size.is_some() {
        existing.chunk_strategy =
            parse_chunk_strategy(req.chunk_strategy.as_deref(), req.chunk_size);
    }
    existing
}

fn parse_chunk_strategy(strategy: Option<&str>, size: Option<usize>) -> ChunkingStrategy {
    let size = size.unwrap_or(512);
    match strategy {
        Some("fixed") => ChunkingStrategy::FixedSize { size },
        Some("recursive") => ChunkingStrategy::Recursive { size },
        Some("token") => ChunkingStrategy::Token { tokens: size },
        Some("sentence") => ChunkingStrategy::Sentence,
        Some("document") => ChunkingStrategy::Document,
        Some("semantic") => ChunkingStrategy::Semantic { threshold: 0.7 },
        _ => ChunkingStrategy::Recursive { size },
    }
}
