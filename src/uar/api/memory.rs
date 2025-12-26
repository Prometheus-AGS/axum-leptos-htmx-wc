use crate::AppState;
use crate::uar::domain::memory::{Memory, MemoryMatch};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SaveMemoryRequest {
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMemoryQuery {
    pub q: String,
    pub agent_id: Option<String>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}

pub async fn save_memory_handler(
    State(state): State<AppState>,
    Json(payload): Json<SaveMemoryRequest>,
) -> impl IntoResponse {
    let persistence = match &state.persistence {
        Some(p) => p,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Persistence not enabled").into_response();
        }
    };

    // Generate embedding
    // VectorMatcher only has embed_batch
    let embedding = match state
        .vector_matcher
        .embed_batch(vec![payload.content.clone()])
        .await
    {
        Ok(mut e) => {
            if let Some(emb) = e.pop() {
                emb
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "No embedding generated".to_string(),
                )
                    .into_response();
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Embedding failed: {}", e),
            )
                .into_response();
        }
    };

    let memory = Memory {
        id: Uuid::new_v4().to_string(),
        agent_id: payload.agent_id, // None is global
        content: payload.content,
        tags: payload.tags.unwrap_or_default(),
        embedding,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    if let Err(e) = persistence.save_memory(&memory).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Save failed: {}", e),
        )
            .into_response();
    }

    Json(json!({
        "status": "success",
        "id": memory.id
    }))
    .into_response()
}

pub async fn search_memory_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchMemoryQuery>,
) -> impl IntoResponse {
    let persistence = match &state.persistence {
        Some(p) => p,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Persistence not enabled").into_response();
        }
    };

    // Generate embedding for query
    let embedding = match state
        .vector_matcher
        .embed_batch(vec![query.q.clone()])
        .await
    {
        Ok(mut e) => {
            if let Some(emb) = e.pop() {
                emb
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "No embedding generated".to_string(),
                )
                    .into_response();
            }
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Embedding failed: {}", e),
            )
                .into_response();
        }
    };

    let limit = query.limit.unwrap_or(10);
    let min_score = query.min_score.unwrap_or(0.0);

    // Agent ID logic:
    // If query.agent_id is Some, search specific agent + global.
    // If None, search ONLY global? Or implied "current agent"?
    // The requirements say "System and per-agent basis".
    // If I omit agent_id in query, I likely want System/Global memories.
    // So passing query.agent_id.as_deref() works (matches PostgresProvider logic).

    let matches = match persistence
        .search_memory(query.agent_id.as_deref(), &embedding, limit, min_score)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search failed: {}", e),
            )
                .into_response();
        }
    };

    Json(matches).into_response()
}
