//! LLM orchestrator with tool loop execution.
//!
//! The orchestrator manages the complete lifecycle of an LLM interaction:
//! 1. Send user message to the LLM
//! 2. Stream the response, detecting tool calls
//! 3. Execute tool calls via MCP
//! 4. Feed tool results back to the LLM
//! 5. Repeat until the model produces a final response
//!
//! # Example
//!
//! ```rust,ignore
//! use axum_leptos_htmx_wc::llm::{Orchestrator, LlmSettings, LlmProtocol};
//! use axum_leptos_htmx_wc::mcp::registry::McpRegistry;
//!
//! let settings = LlmSettings { /* ... */ };
//! let mcp = McpRegistry::load_from_file("mcp.json").await?;
//! let orchestrator = Orchestrator::new(settings, mcp);
//!
//! let stream = orchestrator.chat("Hello, what time is it?").await?;
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use uuid::Uuid;

use crate::mcp::registry::McpRegistry;
use crate::normalized::NormalizedEvent;

use super::{
    ChatCompletionsDriver, LlmDriver, LlmProtocol, LlmRequest, LlmSettings, Message, MessageRole,
    ResponsesDriver, ToolCall, ToolCallFunction,
};

/// Maximum number of tool loop iterations to prevent infinite loops.
const MAX_TOOL_ITERATIONS: usize = 10;

/// Accumulated state for a streaming tool call.
#[derive(Debug, Default, Clone)]
struct ToolCallAccumulator {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

/// LLM orchestrator with tool loop execution.
///
/// The orchestrator wraps an [`LlmDriver`] and adds:
/// - Tool call detection and accumulation
/// - Tool execution via MCP
/// - Automatic tool result feeding
/// - Request ID tracking
#[derive(Clone)]
pub struct Orchestrator {
    settings: LlmSettings,
    mcp: Arc<McpRegistry>,
    driver: Arc<dyn LlmDriver>,
}

impl std::fmt::Debug for Orchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Orchestrator")
            .field("settings", &self.settings)
            .field("mcp", &"McpRegistry")
            .finish()
    }
}

impl Orchestrator {
    /// Create a new orchestrator with the given settings and MCP registry.
    pub fn new(settings: LlmSettings, mcp: Arc<McpRegistry>) -> Self {
        let driver: Arc<dyn LlmDriver> = match settings.protocol {
            LlmProtocol::Responses => Arc::new(ResponsesDriver::new(settings.clone())),
            LlmProtocol::Chat => Arc::new(ChatCompletionsDriver::new(settings.clone())),
            LlmProtocol::Auto => {
                if settings.base_url.contains("openai.com") {
                    Arc::new(ResponsesDriver::new(settings.clone()))
                } else {
                    Arc::new(ChatCompletionsDriver::new(settings.clone()))
                }
            }
        };

        Self {
            settings,
            mcp,
            driver,
        }
    }

    /// Get the LLM settings.
    #[must_use]
    pub fn settings(&self) -> &LlmSettings {
        &self.settings
    }

    /// Get the MCP registry.
    #[must_use]
    pub fn mcp(&self) -> &McpRegistry {
        &self.mcp
    }

    /// Start a chat interaction with the given user message.
    ///
    /// Returns a stream of [`NormalizedEvent`]s that includes:
    /// - `StreamStart` with a unique request ID
    /// - `MessageDelta` for assistant text
    /// - `ToolCallDelta` and `ToolCallComplete` for tool calls
    /// - `ToolResult` after tool execution
    /// - `Done` when complete
    ///
    /// The orchestrator will automatically execute tool calls and feed
    /// results back to the LLM until a final response is produced.
    pub async fn chat(
        &self,
        user_message: &str,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send> {
        self.chat_with_history(vec![Message {
            role: MessageRole::User,
            content: user_message.to_string(),
            tool_call_id: None,
            tool_calls: None,
        }])
        .await
    }

    /// Start a chat interaction with existing message history.
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send> {
        let request_id = Uuid::new_v4().to_string();
        let tools = self.mcp.openai_tools_json();

        let orchestrator = self.clone();
        let messages = messages.clone();

        let stream = async_stream::stream! {
            // Emit stream start
            yield NormalizedEvent::StreamStart {
                request_id: request_id.clone(),
            };

            // Convert messages to JSON for the driver
            let mut message_json: Vec<serde_json::Value> = messages
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();

            let mut iteration = 0;

            loop {
                if iteration >= MAX_TOOL_ITERATIONS {
                    yield NormalizedEvent::Error {
                        message: "Maximum tool loop iterations exceeded".to_string(),
                        code: Some("MAX_ITERATIONS".to_string()),
                    };
                    break;
                }
                iteration += 1;

                let req = LlmRequest {
                    messages: message_json.clone(),
                    tools: tools.clone(),
                };

                // Stream from the driver
                let driver_stream = match orchestrator.driver.stream(req).await {
                    Ok(s) => s,
                    Err(e) => {
                        yield NormalizedEvent::Error {
                            message: e.to_string(),
                            code: None,
                        };
                        break;
                    }
                };

                let mut tool_accumulators: BTreeMap<usize, ToolCallAccumulator> = BTreeMap::new();
                let mut assistant_text = String::new();
                let mut has_tool_calls = false;
                let mut finish_reason: Option<String> = None;

                futures::pin_mut!(driver_stream);

                while let Some(result) = driver_stream.next().await {
                    match result {
                        Ok(event) => {
                            match &event {
                                NormalizedEvent::MessageDelta { text } => {
                                    assistant_text.push_str(text);
                                }
                                NormalizedEvent::ToolCallDelta {
                                    call_index,
                                    id,
                                    name,
                                    arguments_delta,
                                } => {
                                    has_tool_calls = true;
                                    let acc = tool_accumulators.entry(*call_index).or_default();
                                    if acc.id.is_none() {
                                        acc.id = id.clone();
                                    }
                                    if acc.name.is_none() {
                                        acc.name = name.clone();
                                    }
                                    if let Some(delta) = arguments_delta {
                                        acc.arguments.push_str(delta);
                                    }
                                }
                                NormalizedEvent::ToolCallComplete { .. } => {
                                    has_tool_calls = true;
                                    finish_reason = Some("tool_calls".to_string());
                                }
                                NormalizedEvent::Done => {
                                    // Don't yield Done yet if we have tool calls to process
                                    if !has_tool_calls {
                                        yield event;
                                        return;
                                    }
                                    continue;
                                }
                                NormalizedEvent::Error { .. } => {
                                    yield event;
                                    return;
                                }
                                _ => {}
                            }
                            yield event;
                        }
                        Err(e) => {
                            yield NormalizedEvent::Error {
                                message: e.to_string(),
                                code: None,
                            };
                            return;
                        }
                    }
                }

                // If no tool calls, we're done
                if !has_tool_calls || finish_reason.as_deref() != Some("tool_calls") {
                    yield NormalizedEvent::Done;
                    break;
                }

                // Build tool calls from accumulators
                let tool_calls: Vec<ToolCall> = tool_accumulators
                    .values()
                    .filter_map(|acc| {
                        let id = acc.id.clone()?;
                        let name = acc.name.clone()?;
                        Some(ToolCall {
                            id,
                            call_type: "function".to_string(),
                            function: ToolCallFunction {
                                name,
                                arguments: acc.arguments.clone(),
                            },
                        })
                    })
                    .collect();

                if tool_calls.is_empty() {
                    yield NormalizedEvent::Done;
                    break;
                }

                // Add assistant message with tool calls to history
                message_json.push(serde_json::json!({
                    "role": "assistant",
                    "content": if assistant_text.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(assistant_text.clone()) },
                    "tool_calls": tool_calls.iter().map(|tc| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": tc.call_type,
                            "function": {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments
                            }
                        })
                    }).collect::<Vec<_>>()
                }));

                // Execute each tool call and emit results
                for tool_call in &tool_calls {
                    let tool_name = &tool_call.function.name;
                    let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    let (content, success) = match orchestrator.mcp.call_namespaced_tool(tool_name, arguments).await {
                        Ok(result) => {
                            let content = serde_json::to_string(&result).unwrap_or_default();
                            (content, true)
                        }
                        Err(e) => {
                            (format!("Error: {e}"), false)
                        }
                    };

                    // Emit tool result event
                    yield NormalizedEvent::ToolResult {
                        id: tool_call.id.clone(),
                        name: tool_name.clone(),
                        content: content.clone(),
                        success,
                    };

                    // Add tool result to message history
                    message_json.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": content
                    }));
                }

                // Continue the loop to get the next response
            }
        };

        Ok(stream)
    }
}
