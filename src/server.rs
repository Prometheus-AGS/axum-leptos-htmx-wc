use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    routing::{get, get_service, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

use tracing::info;

use crate::AppState;
use crate::config::AppConfig;
use crate::llm::{LlmSettings, Orchestrator};
use crate::mcp::registry::McpRegistry;
use crate::session::SessionStore;
use crate::uar::{
    self,
    defaults::ensure_default_knowledge_base,
    persistence::{
        PersistenceLayer,
        providers::{postgres::PostgresProvider, surreal::SurrealDbProvider},
    },
    rag::{
        chunking::ChunkingStrategy, ingest::IngestService, ingestion_worker::IngestionWorkerPool,
    },
    runtime::{manager::RunManager, matching::vector::VectorMatcher, skills::SkillRegistry},
};

/// Start the Axum server with the provided configuration.
pub async fn start_server(config: Arc<AppConfig>, settings: LlmSettings) -> anyhow::Result<()> {
    info!(
        name: "llm.config.loaded",
        base_url = %settings.base_url,
        model = %settings.model,
        "LLM configuration loaded"
    );

    // Initialize Persistence & RAG
    let mut ingest_service: Option<Arc<IngestService>> = None;
    let vector_matcher = Arc::new(VectorMatcher::new(0.75));

    // Initialize VectorMatcher explicitly (shared)
    if let Err(e) = vector_matcher.initialize().await {
        tracing::error!("Failed to initialize VectorMatcher: {:?}", e);
    }

    // Initialize persistence based on config
    let persistence: Arc<dyn PersistenceLayer> = match config.persistence.provider.as_str() {
        "surrealdb" => {
            let provider = SurrealDbProvider::new(&config.persistence.database_url)
                .await
                .expect("Failed to initialize SurrealDB");
            Arc::new(provider)
        }
        _ => {
            let provider = PostgresProvider::new(&config.persistence.database_url)
                .await
                .expect("Failed to initialize Postgres");
            Arc::new(provider)
        }
    };
    let persistence = Some(persistence.clone());

    // Initialize Ingest Service if persistence is available
    if let Some(p) = &persistence {
        let ingest = Arc::new(IngestService::new(
            p.clone(),
            vector_matcher.clone(),
            ChunkingStrategy::Semantic { threshold: 0.5 },
        ));
        ingest_service = Some(ingest.clone());

        // Spawn File Watcher
        let ingest_svc_clone = ingest.clone();
        tokio::spawn(async move {
            let ingest_dir = std::path::PathBuf::from("/data/ingest");
            if !ingest_dir.exists() {
                let _ = tokio::fs::create_dir_all(&ingest_dir).await;
            }
            if ingest_dir.exists() {
                ingest_svc_clone
                    .watch(ingest_dir, "default".to_string())
                    .await;
            }
        });

        // Ensure default knowledge base exists
        if let Err(e) = ensure_default_knowledge_base(&**p, None).await {
            tracing::error!("Failed to ensure default KB: {:?}", e);
        } else {
            info!("Default knowledge base ensured.");
        }

        info!("Persistence and RAG enabled.");
    }

    // MCP: connect once at startup
    // We update this to include native tools if persistence is present
    let mut mcp_registry = McpRegistry::load_from_file("mcp.json")
        .await
        .unwrap_or_else(|e| panic!("Failed to load MCP servers: {e:?}"));

    if let Some(p) = &persistence {
        // Register Memory Tools
        let save_tool = Arc::new(crate::uar::tools::memory::MemorySaveTool::new(
            p.clone(),
            vector_matcher.clone(),
        ));
        let recall_tool = Arc::new(crate::uar::tools::memory::MemoryRecallTool::new(
            p.clone(),
            vector_matcher.clone(),
        ));

        mcp_registry = mcp_registry
            .with_native_tool(save_tool)
            .with_native_tool(recall_tool);
        info!("Native tools (Memory) registered.");
    }

    let mcp = Arc::new(mcp_registry);

    for (name, _tool) in mcp.tools() {
        info!(name: "mcp.tool.discovered", tool = %name, "MCP tool discovered");
    }

    // Create orchestrator
    let orchestrator = Arc::new(Orchestrator::new(settings.clone(), Arc::clone(&mcp)));

    // Session store
    let sessions = SessionStore::new();

    // Skills initialization
    let mut skills_registry = SkillRegistry::new(None, None);
    if let Err(e) = skills_registry.load_from_dir("skills").await {
        eprintln!("Warning: Failed to load skills: {:?}", e);
    }
    let skills = Arc::new(RwLock::new(skills_registry));

    let run_manager = Arc::new(
        RunManager::new(
            settings.clone(),
            Arc::clone(&mcp),
            sessions.clone(),
            skills.clone(),
            vector_matcher.clone(), // Passed explicitly
            persistence.clone(),    // Passed explicitly
        )
        .await,
    );

    // Initialize Global Rate Limiter
    let rate_limiter = Arc::new(uar::security::rate_limit::AppRateLimiter::new(
        config.resilience.requests_per_second,
        config.resilience.burst_size as u32,
    ));

    let state = AppState {
        mcp,
        orchestrator,
        sessions,
        run_manager,
        ingest_service,
        vector_matcher: vector_matcher.clone(),
        persistence: persistence.clone(),
        rate_limiter,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/", get_service(ServeFile::new("static/index.html")))
        .route("/about", get_service(ServeFile::new("static/about.html")))
        .route("/api/chat", post(api_chat))
        .route("/api/sessions/{id}/messages", get(api_get_messages))
        .nest(
            "/api/uar",
            uar::api::router().with_state(state.run_manager.clone()),
        )
        // Knowledge Base API
        .nest("/api/uar/knowledge-bases", {
            // Initialize ingestion worker pool if persistence available
            let ingestion_pool = if let Some(p) = &persistence {
                if let Some(ingest) = &state.ingest_service {
                    match IngestionWorkerPool::new(
                        0,   // auto-detect CPU count
                        100, // max queue depth
                        ingest.clone(),
                        p.clone(),
                    ) {
                        Ok(pool) => {
                            info!("Ingestion worker pool initialized");
                            Some(Arc::new(pool))
                        }
                        Err(e) => {
                            tracing::error!("Failed to create ingestion pool: {:?}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            uar::api::knowledge::build_router().with_state(Arc::new(
                uar::api::knowledge::KnowledgeApiState {
                    persistence: persistence
                        .clone()
                        .expect("Persistence required for KB API"),
                    vector_matcher: vector_matcher.clone(),
                    ingestion_pool,
                },
            ))
        })
        .route("/api/ingest", post(uar::api::ingest::ingest_handler))
        .route(
            "/api/memory",
            post(uar::api::memory::save_memory_handler)
                .get(uar::api::memory::search_memory_handler),
        )
        .route(
            "/v1/chat/completions",
            post(uar::api::openai::routes::chat_completions),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            uar::security::middleware::auth_middleware,
        ))
        // Apply Timeout Layer if not disabled
        // We use a large timeout if disabled instead of conditional layering to keep types consistent
        .layer(TraceLayer::new_for_http());

    // We can't easily conditionally apply a layer in the chain if types differ.
    // Standard pattern:
    // let app = app.layer(...)
    // if condition { app = app.layer(...) } -> type changes!
    // Axum Router type changes with layers.
    // Actually, TimeoutLayer is a Service layer.

    // Instead of fighting types, we can use `ServiceBuilder` but even then.
    // A common workaround:
    // Configure all layers in a `ServiceBuilder`? No, same type issue.
    //
    // Option A: Use `BoxRoute` / `BoxService` (performance hit).
    // Option B: Always apply a layer, but make it a No-Op? `TimeoutLayer` doesn't have a no-op mode easily.
    // Option C: Middleware refactoring.

    // Actually, for timeouts, we likely ALWAYs want a timeout unless debugging.
    // But user requirement says "can be turned off".

    // Let's use `MapRequest` or similar? No.
    // Let's go with the `stack` approach if possible or accepted boilerplate.

    // Or we just accept the Box overhead? It's fine for this app.
    // `app.boxed()` ?

    // Let's try to just rebuild the router or use `tower::ServiceBuilder`.
    // Actually, `Router` has `layer`.

    // If I do:
    // let app = Router::new()...;
    // let app = if config.enabled { app.layer(Limit) } else { app };
    // This fails because `app` type changes.

    // Solution: `tower::util::OptionLayer`? No, maybe simpler:
    // Just wrap logic in a custom middleware that applies the timeout?
    // `Timeout` is a service, not just middleware fn.

    // Let's stick to adding them unconditionally for now BUT utilize a very large timeout if disabled?
    // "Timeout disabled" -> Duration::MAX?
    // That effectively disables it without changing types.

    let timeout_duration = if config.resilience.timeout_disabled {
        Duration::from_secs(365 * 24 * 60 * 60) // 1 year
    } else {
        Duration::from_secs(30)
    };

    let app = app
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
        .layer(axum::middleware::from_fn(
            move |req: Request, next: Next| {
                let duration = timeout_duration;
                async move {
                    match tokio::time::timeout(duration, next.run(req)).await {
                        Ok(res) => res,
                        Err(_) => {
                            (StatusCode::REQUEST_TIMEOUT, "Request timed out").into_response()
                        }
                    }
                }
            },
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            uar::security::rate_limit::rate_limit_middleware,
        ))
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!(
        name: "server.started",
        address = %addr,
        "Server started"
    );

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// API Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// Request body for chat API.
#[derive(Debug, Deserialize)]
struct ChatRequest {
    /// User message content.
    message: String,
    /// Optional session ID (creates new if not provided).
    #[serde(default)]
    session_id: Option<String>,
}

/// Response from chat API.
#[derive(Debug, Serialize)]
struct ChatResponse {
    /// Session ID for this conversation.
    session_id: String,
    /// URL for the SSE stream.
    stream_url: String,
}

/// POST /api/chat - Start a chat and get stream URL.
async fn api_chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    tracing::info!(
        message = %req.message,
        session_id = ?req.session_id,
        "Received chat request"
    );

    let session_id = if let Some(id) = &req.session_id {
        if id.is_empty() {
            state.sessions.create().id().to_string()
        } else {
            // We just pass it through, RunManager will validate/create
            id.clone()
        }
    } else {
        state.sessions.create().id().to_string()
    };

    // Start Run via UAR
    let run_id = state
        .run_manager
        .start_run(
            uar::defaults::default_agent(),
            req.message,
            Some(session_id.clone()),
            None,
        )
        .await;

    let stream_url = format!("/api/uar/runs/{}/stream", run_id);

    Ok(Json(ChatResponse {
        session_id,
        stream_url,
    }))
}

/// Message DTO for API responses.
#[derive(Debug, Serialize)]
struct MessageDto {
    role: String,
    content: String,
}

/// GET /api/sessions/:id/messages - Get session messages.
async fn api_get_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<MessageDto>>, StatusCode> {
    match state.sessions.get(&id) {
        Some(session) => {
            let messages: Vec<MessageDto> = session
                .messages()
                .iter()
                .map(|m| MessageDto {
                    role: format!("{:?}", m.role).to_lowercase(),
                    content: m.content.to_string(),
                })
                .collect();
            Ok(Json(messages))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
