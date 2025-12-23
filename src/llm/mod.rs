//! LLM driver traits and implementations.
//!
//! This module provides protocol-agnostic abstractions for interacting with
//! Large Language Models, supporting both OpenAI Chat Completions and
//! Responses APIs.
//!
//! # Overview
//!
//! The [`LlmDriver`] trait defines the core streaming interface that all
//! LLM implementations must support. The [`Orchestrator`] builds on top
//! of drivers to provide tool loop execution.
//!
//! # Drivers
//!
//! - [`ChatCompletionsDriver`]: OpenAI Chat Completions API (`/v1/chat/completions`)
//! - [`ResponsesDriver`]: OpenAI Responses API (`/v1/responses`)
//!
//! # Example
//!
//! ```rust,ignore
//! use axum_leptos_htmx_wc::llm::{LlmSettings, LlmProtocol, Orchestrator};
//!
//! let settings = LlmSettings {
//!     base_url: "https://api.openai.com".to_string(),
//!     api_key: Some("sk-...".to_string()),
//!     model: "gpt-4".to_string(),
//!     protocol: LlmProtocol::Chat,
//! };
//! ```

pub mod chat_completions;
pub mod orchestrator;
pub mod responses;

pub use chat_completions::ChatCompletionsDriver;
pub use orchestrator::Orchestrator;
pub use responses::ResponsesDriver;

use crate::normalized::NormalizedEvent;
use futures::Stream;

/// LLM connection and model settings.
#[derive(Debug, Clone)]
pub struct LlmSettings {
    /// Base URL for the LLM API (e.g., `https://api.openai.com`).
    pub base_url: String,
    /// Optional API key for authentication.
    pub api_key: Option<String>,
    /// Model identifier (e.g., `gpt-4`, `claude-3-opus`).
    pub model: String,
    /// Protocol to use for communication.
    pub protocol: LlmProtocol,
}

/// LLM protocol variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LlmProtocol {
    /// Automatically detect protocol based on the provider.
    #[default]
    Auto,
    /// OpenAI Responses API (`/v1/responses`).
    Responses,
    /// OpenAI Chat Completions API (`/v1/chat/completions`).
    Chat,
}

/// A message in a conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    /// Role of the message author.
    pub role: MessageRole,
    /// Content of the message.
    pub content: String,
    /// Optional tool call ID (for tool responses).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional tool calls made by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Role of a message author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System prompt.
    System,
    /// User message.
    User,
    /// Assistant response.
    Assistant,
    /// Tool response.
    Tool,
}

/// A tool call made by the assistant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    pub id: String,
    /// Type of tool (always "function" for now).
    #[serde(rename = "type")]
    pub call_type: String,
    /// Function details.
    pub function: ToolCallFunction,
}

/// Function details in a tool call.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCallFunction {
    /// Function name.
    pub name: String,
    /// Arguments as JSON string.
    pub arguments: String,
}

/// Request to an LLM driver.
#[derive(Debug)]
pub struct LlmRequest {
    /// Conversation messages.
    pub messages: Vec<serde_json::Value>,
    /// Available tools in OpenAI function schema format.
    pub tools: Vec<serde_json::Value>,
}

/// Trait for LLM streaming drivers.
///
/// Implementations of this trait provide streaming access to LLM responses,
/// emitting [`NormalizedEvent`]s as the model generates output.
#[async_trait::async_trait]
pub trait LlmDriver: Send + Sync {
    /// Stream a response from the LLM.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the connection is interrupted.
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>;
}
