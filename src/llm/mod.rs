//! LLM driver traits and implementations.
//!
//! This module provides protocol-agnostic abstractions for interacting with
//! Large Language Models, supporting both `OpenAI` Chat Completions and
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
//! - [`ChatCompletionsDriver`]: `OpenAI` Chat Completions API (`/v1/chat/completions`)
//! - [`ResponsesDriver`]: `OpenAI` Responses API (`/v1/responses`)
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
pub mod provider;
pub mod responses;

pub use chat_completions::ChatCompletionsDriver;
pub use orchestrator::Orchestrator;
pub use provider::Provider;
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
    /// Provider type (auto-detected from `base_url` if not specified).
    pub provider: Provider,
    /// Whether to enable parallel tool calls (provider-dependent).
    pub parallel_tool_calls: Option<bool>,
    /// Azure deployment name (required for Azure `OpenAI`).
    #[allow(dead_code)]
    pub deployment_name: Option<String>,
    /// Azure API version (required for Azure `OpenAI`).
    #[allow(dead_code)]
    pub api_version: Option<String>,
}

/// LLM protocol variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LlmProtocol {
    /// Automatically detect protocol based on the provider.
    #[default]
    Auto,
    /// `OpenAI` Responses API (`/v1/responses`).
    Responses,
    /// `OpenAI` Chat Completions API (`/v1/chat/completions`).
    Chat,
}

/// A message in a conversation.
///
/// Messages can contain either simple text content or multimodal content
/// with images and text parts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    /// Role of the message author.
    pub role: MessageRole,
    /// Content of the message (text or multimodal parts).
    #[serde(flatten)]
    pub content: MessageContent,
    /// Optional tool call ID (for tool responses).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional tool calls made by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Message content - either simple text or multimodal parts.
///
/// This enum allows backward compatibility with text-only messages
/// while supporting the new multimodal content format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content (backward compatible).
    Text { content: String },
    /// Multimodal content with text and image parts.
    Parts { content: Vec<ContentPart> },
}

impl MessageContent {
    /// Create simple text content.
    #[must_use]
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text { content: s.into() }
    }

    /// Create multimodal content from parts.
    #[must_use]
    pub fn parts(parts: Vec<ContentPart>) -> Self {
        Self::Parts { content: parts }
    }

    /// Get the text content (first text part or entire string).
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { content } => Some(content),
            Self::Parts { content } => content.iter().find_map(|p| {
                if let ContentPart::Text { text } = p {
                    Some(text.as_str())
                } else {
                    None
                }
            }),
        }
    }

    /// Get a mutable reference to the text content.
    #[must_use]
    pub fn as_text_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Text { content } => Some(content),
            Self::Parts { .. } => None,
        }
    }

    /// Check if this content contains any images.
    #[must_use]
    pub fn has_images(&self) -> bool {
        matches!(self, Self::Parts { content } if content.iter().any(|p| matches!(p, ContentPart::ImageUrl { .. })))
    }

    /// Check if the content is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text { content } => content.is_empty(),
            Self::Parts { content } => content.is_empty(),
        }
    }
}

impl Default for MessageContent {
    fn default() -> Self {
        Self::text("")
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::text(s)
    }
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_text() {
            Some(text) => write!(f, "{text}"),
            None => write!(f, "[multimodal content]"),
        }
    }
}

/// A content part for multimodal messages.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content.
    Text {
        /// The text content.
        text: String,
    },
    /// Image content (URL or base64 data URL).
    ImageUrl {
        /// Image URL configuration.
        image_url: ImageUrl,
    },
}

impl ContentPart {
    /// Create a text content part.
    #[must_use]
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text { text: s.into() }
    }

    /// Create an image URL content part.
    #[must_use]
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }

    /// Create an image URL content part with detail level.
    #[must_use]
    pub fn image_url_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl {
                url: url.into(),
                detail: Some(detail.into()),
            },
        }
    }
}

/// Image URL configuration for multimodal content.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImageUrl {
    /// Image URL (can be HTTP URL or base64 data URL).
    pub url: String,
    /// Detail level for image processing: "auto", "low", or "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
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
    /// Available tools in `OpenAI` function schema format.
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
