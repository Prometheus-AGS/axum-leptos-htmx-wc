//! Normalized event types for streaming LLM responses.
//!
//! This module defines a unified event model that abstracts over different LLM
//! protocols (Chat Completions, Responses API) and provides consistent streaming
//! events for the client UI.
//!
//! # Event Types
//!
//! The [`NormalizedEvent`] enum covers all possible streaming events:
//! - Message deltas for incremental text output
//! - Tool call lifecycle (delta, complete, result)
//! - Extended model capabilities (thinking, reasoning, citations, memory)
//! - Stream lifecycle (start, done, error)
//!
//! # Example
//!
//! ```rust
//! use axum_leptos_htmx_wc::normalized::{NormalizedEvent, sse_event};
//!
//! let event = NormalizedEvent::MessageDelta {
//!     text: "Hello".to_string(),
//! };
//! let sse = sse_event(&event);
//! assert!(sse.contains("message.delta"));
//! ```

use serde::{Deserialize, Serialize};

/// Citation reference for source attribution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Citation {
    /// Zero-based index of this citation in the response.
    pub index: usize,
    /// URL of the source.
    pub url: String,
    /// Optional title of the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional snippet from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Normalized streaming events emitted by the LLM orchestrator.
///
/// These events provide a unified interface for the client UI regardless
/// of which LLM protocol is used (Chat Completions, Responses API, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum NormalizedEvent {
    // ─────────────────────────────────────────────────────────────────────
    // Stream Lifecycle
    // ─────────────────────────────────────────────────────────────────────
    /// Indicates the start of a new streaming response.
    #[serde(rename = "stream.start")]
    StreamStart {
        /// Unique identifier for this request/response pair.
        request_id: String,
    },

    // ─────────────────────────────────────────────────────────────────────
    // Message Content
    // ─────────────────────────────────────────────────────────────────────
    /// Incremental text delta from the assistant's response.
    #[serde(rename = "message.delta")]
    MessageDelta {
        /// The text fragment to append.
        text: String,
    },

    // ─────────────────────────────────────────────────────────────────────
    // Extended Model Capabilities
    // ─────────────────────────────────────────────────────────────────────
    /// Incremental thinking/internal reasoning delta (for models that expose this).
    #[serde(rename = "thinking.delta")]
    ThinkingDelta {
        /// The thinking text fragment to append.
        text: String,
    },

    /// Incremental reasoning delta (chain-of-thought output).
    #[serde(rename = "reasoning.delta")]
    ReasoningDelta {
        /// The reasoning text fragment to append.
        text: String,
    },

    /// A citation/source reference was added.
    #[serde(rename = "citation.added")]
    CitationAdded(Citation),

    /// Memory/context update from the model.
    #[serde(rename = "memory.update")]
    MemoryUpdate {
        /// Key for the memory entry.
        key: String,
        /// Value to store.
        value: String,
        /// Operation type: "set", "append", or "delete".
        #[serde(default = "default_memory_operation")]
        operation: String,
    },

    // ─────────────────────────────────────────────────────────────────────
    // Tool Calls
    // ─────────────────────────────────────────────────────────────────────
    /// Incremental tool call delta (streaming tool call assembly).
    #[serde(rename = "tool_call.delta")]
    ToolCallDelta {
        /// Index of this tool call in the current batch.
        call_index: usize,
        /// Tool call ID (may arrive in first delta or later).
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// Tool/function name (may arrive in first delta or later).
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// Incremental arguments JSON fragment.
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments_delta: Option<String>,
    },

    /// Tool call is fully assembled and ready for execution.
    #[serde(rename = "tool_call.complete")]
    ToolCallComplete {
        /// Index of this tool call in the current batch.
        call_index: usize,
        /// Tool call ID.
        id: String,
        /// Tool/function name.
        name: String,
        /// Complete arguments as JSON string.
        arguments_json: String,
    },

    /// Result from executing a tool.
    #[serde(rename = "tool_result")]
    ToolResult {
        /// Tool call ID this result corresponds to.
        id: String,
        /// Tool/function name.
        name: String,
        /// Result content (typically JSON).
        content: String,
        /// Whether the tool execution succeeded.
        #[serde(default = "default_true")]
        success: bool,
    },

    // ─────────────────────────────────────────────────────────────────────
    // Errors and Completion
    // ─────────────────────────────────────────────────────────────────────
    /// An error occurred during streaming.
    #[serde(rename = "error")]
    Error {
        /// Error message.
        message: String,
        /// Optional error code for programmatic handling.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },

    /// Stream has completed successfully.
    #[serde(rename = "done")]
    Done,
}

fn default_memory_operation() -> String {
    "set".to_string()
}

fn default_true() -> bool {
    true
}

/// Convert a [`NormalizedEvent`] to an SSE-formatted string.
///
/// The output follows the Server-Sent Events specification with both
/// an `event:` line (for EventSource listeners) and a `data:` line
/// containing the JSON payload.
///
/// # Example
///
/// ```rust
/// use axum_leptos_htmx_wc::normalized::{NormalizedEvent, sse_event};
///
/// let event = NormalizedEvent::Done;
/// let sse = sse_event(&event);
/// assert!(sse.contains("event: done"));
/// ```
pub fn sse_event(evt: &NormalizedEvent) -> String {
    let json = serde_json::to_string(evt).unwrap_or_else(|e| {
        serde_json::json!({ "type": "error", "data": { "message": e.to_string() } }).to_string()
    });

    let event_name = event_name(evt);

    format!("event: {event_name}\ndata: {json}\n\n")
}

/// Get the SSE event name for a [`NormalizedEvent`].
pub fn event_name(evt: &NormalizedEvent) -> &'static str {
    match evt {
        NormalizedEvent::StreamStart { .. } => "stream.start",
        NormalizedEvent::MessageDelta { .. } => "message.delta",
        NormalizedEvent::ThinkingDelta { .. } => "thinking.delta",
        NormalizedEvent::ReasoningDelta { .. } => "reasoning.delta",
        NormalizedEvent::CitationAdded { .. } => "citation.added",
        NormalizedEvent::MemoryUpdate { .. } => "memory.update",
        NormalizedEvent::ToolCallDelta { .. } => "tool_call.delta",
        NormalizedEvent::ToolCallComplete { .. } => "tool_call.complete",
        NormalizedEvent::ToolResult { .. } => "tool_result",
        NormalizedEvent::Error { .. } => "error",
        NormalizedEvent::Done => "done",
    }
}

/// Convert a [`NormalizedEvent`] to an AG-UI compatible SSE event.
///
/// AG-UI events use a different naming convention (`agui.*`) and structure
/// to support the AG-UI protocol while maintaining compatibility.
pub fn agui_sse_event(evt: &NormalizedEvent, request_id: &str) -> String {
    let (event_name, payload) = match evt {
        NormalizedEvent::StreamStart { request_id: rid } => (
            "agui.stream.start",
            serde_json::json!({
                "kind": "stream",
                "phase": "start",
                "request_id": rid
            }),
        ),
        NormalizedEvent::MessageDelta { text } => (
            "agui.message.delta",
            serde_json::json!({
                "kind": "message",
                "phase": "delta",
                "request_id": request_id,
                "delta": { "text": text }
            }),
        ),
        NormalizedEvent::ThinkingDelta { text } => (
            "agui.thinking.delta",
            serde_json::json!({
                "kind": "thinking",
                "phase": "delta",
                "request_id": request_id,
                "delta": { "text": text }
            }),
        ),
        NormalizedEvent::ReasoningDelta { text } => (
            "agui.reasoning.delta",
            serde_json::json!({
                "kind": "reasoning",
                "phase": "delta",
                "request_id": request_id,
                "delta": { "text": text }
            }),
        ),
        NormalizedEvent::CitationAdded(citation) => (
            "agui.citation.added",
            serde_json::json!({
                "kind": "citation",
                "phase": "added",
                "request_id": request_id,
                "citation": citation
            }),
        ),
        NormalizedEvent::MemoryUpdate {
            key,
            value,
            operation,
        } => (
            "agui.memory.update",
            serde_json::json!({
                "kind": "memory",
                "phase": "update",
                "request_id": request_id,
                "key": key,
                "value": value,
                "operation": operation
            }),
        ),
        NormalizedEvent::ToolCallDelta {
            call_index,
            id,
            name,
            arguments_delta,
        } => (
            "agui.tool_call.delta",
            serde_json::json!({
                "kind": "tool_call",
                "phase": "delta",
                "request_id": request_id,
                "call_index": call_index,
                "id": id,
                "name": name,
                "delta": { "arguments": arguments_delta }
            }),
        ),
        NormalizedEvent::ToolCallComplete {
            call_index,
            id,
            name,
            arguments_json,
        } => (
            "agui.tool_call.complete",
            serde_json::json!({
                "kind": "tool_call",
                "phase": "complete",
                "request_id": request_id,
                "call_index": call_index,
                "id": id,
                "name": name,
                "arguments_json": arguments_json
            }),
        ),
        NormalizedEvent::ToolResult {
            id,
            name,
            content,
            success,
        } => (
            "agui.tool_result",
            serde_json::json!({
                "kind": "tool_result",
                "request_id": request_id,
                "id": id,
                "name": name,
                "content": content,
                "success": success
            }),
        ),
        NormalizedEvent::Error { message, code } => (
            "agui.error",
            serde_json::json!({
                "kind": "error",
                "request_id": request_id,
                "message": message,
                "code": code
            }),
        ),
        NormalizedEvent::Done => (
            "agui.done",
            serde_json::json!({
                "kind": "done",
                "request_id": request_id
            }),
        ),
    };

    let json = serde_json::to_string(&payload).unwrap_or_else(|e| {
        serde_json::json!({ "kind": "error", "message": e.to_string() }).to_string()
    });

    format!("event: {event_name}\ndata: {json}\n\n")
}

/// Emit both normalized and AG-UI events for a single [`NormalizedEvent`].
///
/// This is useful for clients that support either protocol.
pub fn dual_sse_event(evt: &NormalizedEvent, request_id: &str) -> String {
    let normalized = sse_event(evt);
    let agui = agui_sse_event(evt, request_id);
    format!("{normalized}{agui}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_delta_serialization() {
        let event = NormalizedEvent::MessageDelta {
            text: "Hello".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("message.delta"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_sse_event_format() {
        let event = NormalizedEvent::Done;
        let sse = sse_event(&event);
        assert!(sse.starts_with("event: done\n"));
        assert!(sse.contains("data: "));
        assert!(sse.ends_with("\n\n"));
    }

    #[test]
    fn test_citation_serialization() {
        let citation = Citation {
            index: 0,
            url: "https://example.com".to_string(),
            title: Some("Example".to_string()),
            snippet: None,
        };
        let event = NormalizedEvent::CitationAdded(citation);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("citation.added"));
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_agui_event_format() {
        let event = NormalizedEvent::MessageDelta {
            text: "test".to_string(),
        };
        let sse = agui_sse_event(&event, "req-123");
        assert!(sse.contains("agui.message.delta"));
        assert!(sse.contains("req-123"));
    }
}
