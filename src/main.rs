//! Axum + Leptos + HTMX + Web Components Server
//!
//! Entry point for the agentic streaming LLM application.

// Allow pedantic clippy warnings that don't add value for this codebase
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::unused_async)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::default_trait_access)]

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

use llm::{LlmProtocol, LlmSettings, Message, MessageRole, Orchestrator, Provider, ToolCall};
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
        .route("/api/generate-title", post(api_generate_title))
        .route("/api/models", get(api_get_models))
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
    
    <!-- HTMX and Extensions (local) -->
    <script src="/static/vendor/htmx-2.0.8.min.js"></script>
    <script src="/static/vendor/htmx-json-enc.js"></script>
    <script src="/static/vendor/htmx-sse.js"></script>
    <script defer src="/static/vendor/alpine.min.js"></script>
    
    <!-- Application bundle -->
    <script type="module" src="/static/main.js"></script>
    <link rel="stylesheet" href="/static/app.css">
</head>
<body class="min-h-screen bg-background text-textPrimary antialiased">
    <div id="app-shell" class="flex flex-col h-screen overflow-hidden">
        <header class="sticky top-0 z-50 w-full bg-surfaceContainer backdrop-blur shadow-sm shrink-0">
            <div class="container mx-auto flex h-14 md:h-16 items-center justify-between px-4 md:px-6 max-w-5xl">
                <a href="/" class="flex items-center gap-2 md:gap-3 font-semibold hover:opacity-80 transition-opacity">
                    <svg class="h-5 w-5 md:h-6 md:w-6 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                    </svg>
                    <span class="text-base md:text-lg">Prometheus</span>
                </a>
                <div class="flex items-center gap-1 md:gap-2">
                    <nav class="flex items-center gap-1" hx-boost="true">
                        <a href="/" class="px-3 py-2 rounded-xl text-sm text-textSecondary hover:text-textPrimary hover:bg-surface transition-all">Chat</a>
                        <a href="/about" class="px-3 py-2 rounded-xl text-sm text-textSecondary hover:text-textPrimary hover:bg-surface transition-all">About</a>
                    </nav>
                    <theme-switcher></theme-switcher>
                </div>
            </div>
        </header>
        
        <main id="app" class="flex-1 overflow-y-auto container mx-auto px-4 md:px-6 py-4 md:py-8 max-w-5xl">
            {content}
        </main>
        
        <footer class="bg-surfaceContainer py-3 md:py-6 shrink-0 hidden md:block">
            <div class="container mx-auto px-4 md:px-6 max-w-5xl">
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
    <div class="flex h-full md:h-[calc(100vh-12rem)]">
        <!-- Conversation Sidebar -->
        <conversation-sidebar></conversation-sidebar>
        
        <!-- Main Chat Area -->
        <div class="chat-shell flex flex-col flex-1 bg-surface md:rounded-3xl overflow-hidden md:shadow-lg" style="margin-left: 288px;">
            <header class="flex items-center justify-between px-4 md:px-6 py-3 md:py-4 bg-surfaceContainer shrink-0">
                <div class="flex items-center gap-2 md:gap-3">
                    <svg class="h-5 w-5 md:h-6 md:w-6 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                    </svg>
                    <h2 class="font-semibold text-base md:text-lg">AI Assistant</h2>
                </div>
                <div class="flex items-center gap-3">
                    <token-counter
                        x-bind:input-tokens="$store.chat.tokenUsage.input"
                        x-bind:output-tokens="$store.chat.tokenUsage.output"
                        x-bind:context-limit="$store.chat.tokenUsage.limit"
                        x-bind:cost="$store.chat.tokenUsage.cost"
                        x-bind:is-estimate="$store.chat.tokenUsage.isEstimate"
                        model-id="gpt-4o">
                    </token-counter>
                </div>
            </header>
            
            <div class="flex-1 overflow-y-auto overflow-x-hidden">
                <chat-stream class="block" stream-url="/stream"></chat-stream>
            </div>
        
        <div class="p-3 md:p-5 bg-surfaceContainer shrink-0">
            <form 
                class="flex gap-2 md:gap-3"
                hx-post="/api/chat"
                hx-trigger="submit"
                hx-swap="none"
                hx-ext="json-enc"
                hx-on--before-request="
                    const msg = this.querySelector('[name=message]').value;
                    const chatStream = document.querySelector('chat-stream');
                    
                    // Add user message to UI immediately
                    if (chatStream && msg.trim()) {
                        chatStream.addUserMessage(msg);
                    }
                    
                    // Set session_id from Alpine store if exists
                    const Alpine = window.Alpine;
                    if (Alpine) {
                        const chatStore = Alpine.store('chat');
                        if (chatStore && chatStore.sessionId) {
                            this.querySelector('[name=session_id]').value = chatStore.sessionId;
                        }
                    }
                "
                hx-on--after-request="
                    const response = JSON.parse(event.detail.xhr.response);
                    const chatStream = document.querySelector('chat-stream');
                    
                    // Update session ID in Alpine store
                    const Alpine = window.Alpine;
                    if (Alpine && response.session_id) {
                        const chatStore = Alpine.store('chat');
                        if (chatStore) {
                            chatStore.sessionId = response.session_id;
                        }
                    }
                    
                    // Start streaming (user message already added)
                    if (chatStream) {
                        chatStream.startStream(event.detail.xhr.response);
                    }
                    
                    this.reset();
                "
                x-data="{ message: '' }"
            >
                <!-- Hidden input for session_id -->
                <input type="hidden" name="session_id" x-bind:value="$store.chat?.sessionId || ''">
                
                <div class="flex-1">
                    <textarea
                        name="message"
                        placeholder="Type your message..."
                        class="w-full min-h-[44px] md:min-h-[48px] max-h-[120px] md:max-h-[200px] px-4 md:px-5 py-3 md:py-3.5 rounded-xl md:rounded-2xl bg-surface text-textPrimary placeholder:text-textMuted resize-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-surfaceContainer transition-shadow text-sm md:text-base"
                        rows="1"
                        x-model="message"
                        x-on:keydown.enter.prevent="if (!$event.shiftKey && message.trim()) { $el.form.requestSubmit() }"
                        x-on:input="$el.style.height = 'auto'; $el.style.height = Math.min($el.scrollHeight, window.innerWidth < 768 ? 120 : 200) + 'px'"
                        required
                    ></textarea>
                </div>
                <button 
                    type="submit"
                    class="shrink-0 h-11 w-11 md:h-12 md:w-12 rounded-xl md:rounded-2xl bg-primary text-white hover:bg-primaryMuted active:scale-95 flex items-center justify-center transition-all shadow-md hover:shadow-lg"
                    :disabled="!message.trim()"
                    :class="!message.trim() && 'opacity-50 cursor-not-allowed'"
                >
                    <svg class="h-4 w-4 md:h-5 md:w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="22" y1="2" x2="11" y2="13"/>
                        <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                    </svg>
                </button>
            </form>
            <p class="text-xs text-textMuted mt-2 md:mt-3 text-center hidden md:block">Press Enter to send, Shift+Enter for new line</p>
        </div>
    </div>
    "#
}

/// About page content.
fn about_content() -> &'static str {
    r#"
    <div class="space-y-6">
        <div class="rounded-3xl bg-surface p-8 shadow-lg">
            <h1 class="text-2xl font-bold mb-4">About Prometheus</h1>
            <p class="text-textMuted mb-8">
                Prometheus is an agentic streaming LLM application that demonstrates
                a modern architecture for building AI-powered applications.
            </p>
            
            <div class="grid gap-4 md:grid-cols-2">
                <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                    <h3 class="font-semibold mb-2">🔧 Tool-First Design</h3>
                    <p class="text-sm text-textMuted">Always-on tool use with MCP integration for dynamic tool discovery and execution.</p>
                </div>
                <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                    <h3 class="font-semibold mb-2">⚡ Streaming Native</h3>
                    <p class="text-sm text-textMuted">First-class streaming for tokens, tool calls, and results with AG-UI events.</p>
                </div>
                <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                    <h3 class="font-semibold mb-2">🌐 HTML-Centric</h3>
                    <p class="text-sm text-textMuted">HTMX + Web Components + Alpine.js for a lightweight, inspectable UI.</p>
                </div>
                <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                    <h3 class="font-semibold mb-2">📦 Tauri Ready</h3>
                    <p class="text-sm text-textMuted">No CDN scripts, local assets only - runs as web, desktop, or mobile.</p>
                </div>
            </div>
            
            <div class="mt-8">
                <a href="/" class="inline-flex items-center justify-center h-12 px-6 rounded-2xl bg-primary text-white hover:bg-primaryMuted active:scale-95 font-medium transition-all shadow-md hover:shadow-lg">
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

/// Request for title generation.
#[derive(Debug, Deserialize)]
struct GenerateTitleRequest {
    /// First user message in the conversation.
    message: String,
}

/// Response from title generation.
#[derive(Debug, Serialize)]
struct GenerateTitleResponse {
    /// Generated title.
    title: String,
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

    let session = if let Some(id) = &req.session_id {
        tracing::debug!(session_id = %id, "Using existing session");
        state.sessions.get_or_create(id)
    } else {
        let session = state.sessions.create();
        tracing::debug!(session_id = %session.id(), "Created new session");
        session
    };

    // Add user message to session
    session.add_user_message(&req.message);
    tracing::debug!(
        session_id = %session.id(),
        message_count = session.message_count(),
        "Added user message to session"
    );

    let session_id = session.id().to_string();
    let stream_url = format!("/api/chat/stream?session_id={session_id}");

    tracing::info!(
        session_id = %session_id,
        stream_url = %stream_url,
        "Chat request processed, returning stream URL"
    );

    Ok(Json(ChatResponse {
        session_id,
        stream_url,
    }))
}

/// POST /api/generate-title - Generate a concise title for a conversation.
async fn api_generate_title(
    State(state): State<AppState>,
    Json(req): Json<GenerateTitleRequest>,
) -> Result<Json<GenerateTitleResponse>, (StatusCode, String)> {
    tracing::info!(
        message_length = req.message.len(),
        "Generating conversation title"
    );

    // Create a simple prompt for title generation
    let prompt = format!(
        "Generate a concise 3-6 word title for a conversation that starts with this message: \"{}\"\n\nRespond with ONLY the title, no quotes, no explanation.",
        req.message.chars().take(200).collect::<String>()
    );

    let messages = vec![Message {
        role: MessageRole::User,
        content: prompt,
        tool_call_id: None,
        tool_calls: None,
    }];

    // Use non-streaming LLM call
    match state.orchestrator.chat_non_streaming(messages).await {
        Ok(title) => {
            let trimmed = title.trim().trim_matches('"').to_string();
            tracing::info!(title = %trimmed, "Generated title");
            Ok(Json(GenerateTitleResponse { title: trimmed }))
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to generate title");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate title: {e}"),
            ))
        }
    }
}

/// GET /api/chat/stream - SSE stream for chat responses.
async fn api_chat_stream(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
) -> Response {
    tracing::info!(
        session_id = %query.session_id,
        has_message = query.message.is_some(),
        "Starting SSE stream"
    );

    let session = if let Some(s) = state.sessions.get(&query.session_id) { s } else {
        tracing::error!(session_id = %query.session_id, "Session not found");
        return single_error_sse("Session not found");
    };

    // If a message was provided, add it
    if let Some(msg) = &query.message
        && !msg.is_empty() {
            session.add_user_message(msg);
            tracing::debug!(
                session_id = %query.session_id,
                message = %msg,
                "Added message from query parameter"
            );
        }

    let messages = session.messages_with_system();
    let request_id = uuid::Uuid::new_v4().to_string();
    let orchestrator = Arc::clone(&state.orchestrator);

    tracing::info!(
        session_id = %query.session_id,
        request_id = %request_id,
        message_count = messages.len(),
        "Starting LLM orchestration"
    );

    // Log full message history
    for (idx, msg) in messages.iter().enumerate() {
        tracing::debug!(
            request_id = %request_id,
            message_index = idx,
            role = ?msg.role,
            content_length = msg.content.len(),
            has_tool_calls = msg.tool_calls.is_some(),
            has_tool_call_id = msg.tool_call_id.is_some(),
            "Message in history"
        );
        tracing::trace!(
            request_id = %request_id,
            message_index = idx,
            content = %msg.content,
            "Full message content"
        );
    }

    let sse_stream = async_stream::stream! {
        let stream = match orchestrator.chat_with_history(messages).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "Failed to start orchestrator"
                );
                let err = NormalizedEvent::Error {
                    message: e.to_string(),
                    code: None,
                };
                yield Ok::<String, std::convert::Infallible>(dual_sse_event(&err, &request_id));
                yield Ok::<String, std::convert::Infallible>(sse_event(&NormalizedEvent::Done));
                return;
            }
        };

        tracing::debug!(request_id = %request_id, "Orchestrator stream started");

        // Accumulate assistant response
        let mut assistant_content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool_calls: std::collections::BTreeMap<usize, (Option<String>, Option<String>, String)> = std::collections::BTreeMap::new();

        futures::pin_mut!(stream);
        while let Some(event) = stream.next().await {
            // Log and accumulate events
            match &event {
                NormalizedEvent::StreamStart { request_id: rid } => {
                    tracing::info!(request_id = %rid, "Stream started");
                }
                NormalizedEvent::MessageDelta { text } => {
                    assistant_content.push_str(text);
                    tracing::trace!(request_id = %request_id, delta_length = text.len(), "Message delta");
                }
                NormalizedEvent::ToolCallDelta { call_index, id, name, arguments_delta } => {
                    let entry = current_tool_calls.entry(*call_index).or_insert((None, None, String::new()));
                    if let Some(id) = id {
                        entry.0 = Some(id.clone());
                    }
                    if let Some(name) = name {
                        entry.1 = Some(name.clone());
                    }
                    if let Some(args) = arguments_delta {
                        entry.2.push_str(args);
                    }
                    tracing::debug!(
                        request_id = %request_id,
                        call_index = call_index,
                        id = ?id,
                        name = ?name,
                        "Tool call delta"
                    );
                }
                NormalizedEvent::ToolCallComplete { call_index, id, name, arguments_json } => {
                    tool_calls.push(ToolCall {
                        id: id.clone(),
                        call_type: "function".to_string(),
                        function: crate::llm::ToolCallFunction {
                            name: name.clone(),
                            arguments: arguments_json.clone(),
                        },
                    });
                    tracing::info!(
                        request_id = %request_id,
                        call_index = call_index,
                        id = %id,
                        name = %name,
                        args_length = arguments_json.len(),
                        "Tool call complete"
                    );
                    tracing::debug!(
                        request_id = %request_id,
                        call_index = call_index,
                        arguments = %arguments_json,
                        "Tool call arguments"
                    );
                }
                NormalizedEvent::ToolResult { id, name, content, success } => {
                    tracing::info!(
                        request_id = %request_id,
                        tool_id = %id,
                        tool_name = %name,
                        success = success,
                        result_length = content.len(),
                        "Tool result"
                    );
                    tracing::debug!(
                        request_id = %request_id,
                        tool_id = %id,
                        result = %content,
                        "Tool result content"
                    );
                }
                NormalizedEvent::Error { message, code } => {
                    tracing::error!(
                        request_id = %request_id,
                        error = %message,
                        code = ?code,
                        "Stream error"
                    );
                }
                NormalizedEvent::Done => {
                    // Save assistant response to session
                    if !assistant_content.is_empty() || !tool_calls.is_empty() {
                        let msg = Message {
                            role: MessageRole::Assistant,
                            content: assistant_content.clone(),
                            tool_call_id: None,
                            tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls.clone()) },
                        };
                        session.add_message(msg);
                        tracing::info!(
                            request_id = %request_id,
                            session_id = %query.session_id,
                            content_length = assistant_content.len(),
                            tool_calls_count = tool_calls.len(),
                            "Saved assistant response to session"
                        );
                    }
                    tracing::info!(request_id = %request_id, "Stream complete");
                }
                _ => {
                    // Handle other event types (ThinkingDelta, ReasoningDelta, CitationAdded, MemoryUpdate)
                    tracing::trace!(request_id = %request_id, event_type = ?event, "Other event type");
                }
            }

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

/// GET /api/models - Proxy request to models.dev API to avoid CORS issues.
async fn api_get_models() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("Proxying request to models.dev API");
    
    let client = reqwest::Client::new();
    let response = client
        .get("https://models.dev/api.json")
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch from models.dev");
            (StatusCode::BAD_GATEWAY, format!("Failed to fetch models: {e}"))
        })?;
    
    if !response.status().is_success() {
        let status = response.status();
        tracing::error!(status = %status, "models.dev returned error status");
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("models.dev returned status: {status}"),
        ));
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to parse models.dev response");
            (StatusCode::BAD_GATEWAY, format!("Failed to parse response: {e}"))
        })?;
    
    tracing::info!("Successfully proxied models.dev API response");
    Ok(Json(data))
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

    // Auto-detect provider from base URL
    let mut provider = Provider::detect_from_url(&base_url);

    // Load Azure-specific settings if needed
    let deployment_name = std::env::var("AZURE_DEPLOYMENT_NAME").ok();
    let api_version = std::env::var("AZURE_API_VERSION").ok();

    // Update provider with Azure deployment info if provided
    if let Provider::AzureOpenAI { .. } = &provider
        && let Some(deployment) = &deployment_name {
            provider = Provider::AzureOpenAI {
                deployment_name: deployment.clone(),
                api_version: api_version.clone().unwrap_or_else(|| "2024-08-01-preview".to_string()),
            };
        }

    // Load optional parallel tool calls setting
    let parallel_tool_calls = std::env::var("LLM_PARALLEL_TOOLS")
        .ok()
        .and_then(|s| s.parse().ok());

    Ok(LlmSettings {
        base_url,
        api_key,
        model,
        protocol,
        provider,
        parallel_tool_calls,
        deployment_name,
        api_version,
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
