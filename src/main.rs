//! Axum + Leptos + HTMX + Web Components Server
//!
//! Entry point for the agentic streaming LLM application.

use mimalloc::MiMalloc;

/// Global allocator for improved performance (M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod llm;
mod mcp;
mod normalized;
mod session;
mod ui;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use dotenvy::dotenv;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use llm::{LlmProtocol, LlmSettings, Orchestrator};
use mcp::registry::McpRegistry;
use normalized::{NormalizedEvent, dual_sse_event, sse_event};
use session::SessionStore;

/// Application state shared across all handlers.
#[derive(Clone)]
struct AppState {
    /// MCP server registry for tool discovery and execution.
    #[allow(dead_code)]
    mcp: Arc<McpRegistry>,
    /// LLM orchestrator for chat interactions.
    orchestrator: Arc<Orchestrator>,
    /// Session store for conversation management.
    sessions: SessionStore,
}

#[tokio::main]
async fn main() {
    // Initialize tracing (M-LOG-STRUCTURED)
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // Load .env (if present)
    let _ = dotenv();

    // Load LLM settings
    let settings = match load_llm_settings() {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("Configuration error: {msg}");
            std::process::exit(1);
        }
    };

    info!(
        name: "llm.config.loaded",
        base_url = %settings.base_url,
        model = %settings.model,
        "LLM configuration loaded"
    );

    // MCP: connect once at startup
    let mcp = Arc::new(
        McpRegistry::load_from_file("mcp.json")
            .await
            .unwrap_or_else(|e| panic!("Failed to load MCP servers: {e:?}")),
    );

    for (name, _tool) in mcp.tools() {
        info!(name: "mcp.tool.discovered", tool = %name, "MCP tool discovered");
    }

    // Create orchestrator
    let orchestrator = Arc::new(Orchestrator::new(settings, Arc::clone(&mcp)));

    // Session store
    let sessions = SessionStore::new();

    let state = AppState {
        mcp,
        orchestrator,
        sessions,
    };

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/chat", post(api_chat))
        .route("/api/chat/stream", get(api_chat_stream))
        .route("/api/sessions", get(api_list_sessions))
        .route("/api/sessions", post(api_create_session))
        .route("/api/sessions/{id}", get(api_get_session))
        .route("/api/sessions/{id}", axum::routing::delete(api_delete_session))
        .route("/api/sessions/{id}/messages", get(api_get_messages))
        // Legacy streaming endpoint (for backward compatibility)
        .route("/stream", get(legacy_stream_chat))
        // HTML pages
        .route("/", get(index_handler))
        .route("/about", get(about_handler))
        // Static assets
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!(
        name: "server.started",
        address = "http://127.0.0.1:3000",
        "Server started"
    );

    axum::serve(listener, app).await.unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// HTML Page Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// Generate the HTML shell for the application.
fn html_shell(title: &str, content: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="Agentic Streaming LLM Application">
    <title>{title} - Prometheus</title>
    
    <!-- HTMX and Alpine (local) -->
    <script src="/static/vendor/htmx-2.0.8.min.js"></script>
    <script defer src="/static/vendor/alpine.min.js"></script>
    
    <!-- Application bundle -->
    <script type="module" src="/static/main.js"></script>
    <link rel="stylesheet" href="/static/app.css">
</head>
<body class="min-h-screen bg-background text-textPrimary antialiased">
    <div id="app-shell" class="flex flex-col min-h-screen">
        <header class="sticky top-0 z-50 w-full border-b border-panelBorder bg-background/95 backdrop-blur">
            <div class="container mx-auto flex h-14 items-center justify-between px-4 max-w-5xl">
                <a href="/" class="flex items-center gap-2 font-semibold">
                    <svg class="h-5 w-5 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                    </svg>
                    <span class="text-lg">Prometheus</span>
                </a>
                <nav class="flex items-center gap-6" hx-boost="true">
                    <a href="/" class="text-sm text-textMuted hover:text-textPrimary transition-colors">Chat</a>
                    <a href="/about" class="text-sm text-textMuted hover:text-textPrimary transition-colors">About</a>
                </nav>
            </div>
        </header>
        
        <main id="app" class="flex-1 container mx-auto px-4 py-6 max-w-5xl">
            {content}
        </main>
        
        <footer class="border-t border-panelBorder py-4">
            <div class="container mx-auto px-4 max-w-5xl">
                <p class="text-xs text-textMuted text-center">
                    Powered by Axum + Leptos + HTMX + Web Components
                </p>
            </div>
        </footer>
    </div>
</body>
</html>"#)
}

/// Chat page content.
fn chat_content() -> &'static str {
    r#"
    <div class="chat-shell flex flex-col h-[calc(100vh-10rem)] bg-panel border border-panelBorder rounded-2xl overflow-hidden">
        <header class="flex items-center justify-between px-4 py-3 border-b border-panelBorder bg-panel/50">
            <div class="flex items-center gap-2">
                <svg class="h-5 w-5 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                </svg>
                <h2 class="font-semibold text-lg">AI Assistant</h2>
            </div>
            <span id="chat-status" class="text-xs bg-panel px-2 py-0.5 rounded-full border border-panelBorder">Ready</span>
        </header>
        
        <div class="flex-1 overflow-hidden">
            <chat-stream class="block h-full" stream-url="/stream"></chat-stream>
        </div>
        
        <div class="border-t border-panelBorder p-4 bg-panel/50">
            <form 
                class="flex gap-2"
                hx-post="/api/chat"
                hx-trigger="submit"
                hx-swap="none"
                x-data="{ message: '' }"
            >
                <div class="flex-1">
                    <textarea
                        name="message"
                        placeholder="Type your message..."
                        class="w-full min-h-[44px] max-h-[200px] px-4 py-3 rounded-xl border border-panelBorder bg-background text-textPrimary placeholder:text-textMuted resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                        rows="1"
                        x-model="message"
                        required
                    ></textarea>
                </div>
                <button 
                    type="submit"
                    class="shrink-0 h-11 w-11 rounded-xl bg-primary text-white hover:bg-primaryMuted flex items-center justify-center"
                >
                    <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="22" y1="2" x2="11" y2="13"/>
                        <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                    </svg>
                </button>
            </form>
            <p class="text-xs text-textMuted mt-2 text-center">Press Enter to send</p>
        </div>
    </div>
    "#
}

/// About page content.
fn about_content() -> &'static str {
    r#"
    <div class="space-y-6">
        <div class="rounded-xl border border-panelBorder bg-panel p-6">
            <h1 class="text-2xl font-bold mb-4">About Prometheus</h1>
            <p class="text-textMuted mb-6">
                Prometheus is an agentic streaming LLM application that demonstrates
                a modern architecture for building AI-powered applications.
            </p>
            
            <div class="grid gap-4 md:grid-cols-2">
                <div class="p-4 rounded-lg border border-panelBorder bg-panel/50">
                    <h3 class="font-semibold mb-2">🔧 Tool-First Design</h3>
                    <p class="text-sm text-textMuted">Always-on tool use with MCP integration for dynamic tool discovery and execution.</p>
                </div>
                <div class="p-4 rounded-lg border border-panelBorder bg-panel/50">
                    <h3 class="font-semibold mb-2">⚡ Streaming Native</h3>
                    <p class="text-sm text-textMuted">First-class streaming for tokens, tool calls, and results with AG-UI events.</p>
                </div>
                <div class="p-4 rounded-lg border border-panelBorder bg-panel/50">
                    <h3 class="font-semibold mb-2">🌐 HTML-Centric</h3>
                    <p class="text-sm text-textMuted">HTMX + Web Components + Alpine.js for a lightweight, inspectable UI.</p>
                </div>
                <div class="p-4 rounded-lg border border-panelBorder bg-panel/50">
                    <h3 class="font-semibold mb-2">📦 Tauri Ready</h3>
                    <p class="text-sm text-textMuted">No CDN scripts, local assets only - runs as web, desktop, or mobile.</p>
                </div>
            </div>
            
            <div class="mt-6">
                <a href="/" class="inline-flex items-center justify-center h-10 px-4 rounded-lg bg-primary text-white hover:bg-primaryMuted font-medium">
                    Start Chatting
                </a>
            </div>
        </div>
    </div>
    "#
}

/// Index page handler.
async fn index_handler() -> impl IntoResponse {
    Html(html_shell("Chat", chat_content()))
}

/// About page handler.
async fn about_handler() -> impl IntoResponse {
    Html(html_shell("About", about_content()))
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

/// Query parameters for stream endpoint.
#[derive(Debug, Deserialize)]
struct StreamQuery {
    /// Session ID.
    session_id: String,
    /// Optional message to send (if not already added).
    #[serde(default)]
    message: Option<String>,
}

/// POST /api/chat - Start a chat and get stream URL.
async fn api_chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let session = match &req.session_id {
        Some(id) => state.sessions.get_or_create(id),
        None => state.sessions.create(),
    };

    // Add user message to session
    session.add_user_message(&req.message);

    let session_id = session.id().to_string();
    let stream_url = format!("/api/chat/stream?session_id={session_id}");

    Ok(Json(ChatResponse {
        session_id,
        stream_url,
    }))
}

/// GET /api/chat/stream - SSE stream for chat responses.
async fn api_chat_stream(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
) -> Response {
    let session = match state.sessions.get(&query.session_id) {
        Some(s) => s,
        None => {
            return single_error_sse("Session not found");
        }
    };

    // If a message was provided, add it
    if let Some(msg) = &query.message {
        if !msg.is_empty() {
            session.add_user_message(msg);
        }
    }

    let messages = session.messages_with_system();
    let request_id = uuid::Uuid::new_v4().to_string();
    let orchestrator = Arc::clone(&state.orchestrator);

    let sse_stream = async_stream::stream! {
        let stream = match orchestrator.chat_with_history(messages).await {
            Ok(s) => s,
            Err(e) => {
                let err = NormalizedEvent::Error {
                    message: e.to_string(),
                    code: None,
                };
                yield Ok::<String, std::convert::Infallible>(dual_sse_event(&err, &request_id));
                yield Ok::<String, std::convert::Infallible>(sse_event(&NormalizedEvent::Done));
                return;
            }
        };

        futures::pin_mut!(stream);
        while let Some(event) = stream.next().await {
            yield Ok::<String, std::convert::Infallible>(dual_sse_event(&event, &request_id));
        }
    };

    let body = axum::body::Body::from_stream(sse_stream);
    build_sse_response(body)
}

/// Session info for listing.
#[derive(Debug, Serialize)]
struct SessionInfo {
    id: String,
    message_count: usize,
}

/// GET /api/sessions - List all sessions.
async fn api_list_sessions(State(state): State<AppState>) -> Json<Vec<SessionInfo>> {
    let sessions: Vec<SessionInfo> = state
        .sessions
        .list_ids()
        .iter()
        .filter_map(|id| {
            state.sessions.get(id).map(|s| SessionInfo {
                id: id.clone(),
                message_count: s.message_count(),
            })
        })
        .collect();

    Json(sessions)
}

/// POST /api/sessions - Create a new session.
async fn api_create_session(State(state): State<AppState>) -> Json<SessionInfo> {
    let session = state.sessions.create();
    Json(SessionInfo {
        id: session.id().to_string(),
        message_count: 0,
    })
}

/// GET /api/sessions/:id - Get session details.
async fn api_get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionInfo>, StatusCode> {
    match state.sessions.get(&id) {
        Some(session) => Ok(Json(SessionInfo {
            id: session.id().to_string(),
            message_count: session.message_count(),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// DELETE /api/sessions/:id - Delete a session.
async fn api_delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    match state.sessions.remove(&id) {
        Some(_) => StatusCode::NO_CONTENT,
        None => StatusCode::NOT_FOUND,
    }
}

/// Message DTO for API responses.
#[derive(Debug, Serialize)]
struct MessageDto {
    role: String,
    content: String,
}

/// GET /api/sessions/:id/messages - Get session messages (HTMX fragment).
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
                    content: m.content.clone(),
                })
                .collect();
            Ok(Json(messages))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy Endpoint
// ─────────────────────────────────────────────────────────────────────────────

/// GET /stream - Legacy streaming endpoint for backward compatibility.
async fn legacy_stream_chat(State(state): State<AppState>) -> Response {
    // Create a temporary session
    let session = state.sessions.create();
    session.add_user_message("Say hello, then call a tool if useful.");

    let messages = session.messages_with_system();
    let request_id = uuid::Uuid::new_v4().to_string();
    let orchestrator = Arc::clone(&state.orchestrator);

    let sse_stream = async_stream::stream! {
        let stream = match orchestrator.chat_with_history(messages).await {
            Ok(s) => s,
            Err(e) => {
                let err = NormalizedEvent::Error {
                    message: e.to_string(),
                    code: None,
                };
                yield Ok::<String, std::convert::Infallible>(dual_sse_event(&err, &request_id));
                yield Ok::<String, std::convert::Infallible>(sse_event(&NormalizedEvent::Done));
                return;
            }
        };

        futures::pin_mut!(stream);
        while let Some(event) = stream.next().await {
            yield Ok::<String, std::convert::Infallible>(dual_sse_event(&event, &request_id));
        }
    };

    let body = axum::body::Body::from_stream(sse_stream);
    build_sse_response(body)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn load_llm_settings() -> Result<LlmSettings, String> {
    let base_url = std::env::var("LLM_BASE_URL")
        .map_err(|_| "Missing required env var: LLM_BASE_URL".to_string())?;
    if base_url.trim().is_empty() {
        return Err("LLM_BASE_URL cannot be empty".to_string());
    }

    let model = std::env::var("LLM_MODEL")
        .map_err(|_| "Missing required env var: LLM_MODEL".to_string())?;
    if model.trim().is_empty() {
        return Err("LLM_MODEL cannot be empty".to_string());
    }

    let api_key = std::env::var("LLM_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let protocol = match std::env::var("LLM_PROTOCOL")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase()
        .as_str()
    {
        "responses" => LlmProtocol::Responses,
        "chat" => LlmProtocol::Chat,
        _ => LlmProtocol::Auto,
    };

    Ok(LlmSettings {
        base_url,
        api_key,
        model,
        protocol,
    })
}

fn single_error_sse(message: &str) -> Response {
    let err = NormalizedEvent::Error {
        message: message.to_string(),
        code: None,
    };
    let done = NormalizedEvent::Done;

    let payload = format!("{}{}", sse_event(&err), sse_event(&done));
    let body = axum::body::Body::from(payload);
    build_sse_response(body)
}

fn build_sse_response(body: axum::body::Body) -> Response {
    let mut resp = Response::new(body);
    let h = resp.headers_mut();
    h.insert("Content-Type", "text/event-stream".parse().unwrap());
    h.insert("Cache-Control", "no-cache".parse().unwrap());
    h.insert("Connection", "keep-alive".parse().unwrap());
    h.insert("X-Accel-Buffering", "no".parse().unwrap());
    resp
}
