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

#[allow(clippy::missing_fields_in_debug)]
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
    #[allow(dead_code)]
    pub fn new(settings: LlmSettings, mcp: Arc<McpRegistry>) -> Self {
        let driver: Arc<dyn LlmDriver> = match settings.protocol {
            LlmProtocol::Responses => Arc::new(ResponsesDriver::new(settings.clone())),
            LlmProtocol::Chat => Arc::new(ChatCompletionsDriver::new(settings.clone())),
            LlmProtocol::Auto => {
                // Default to Chat Completions API as it's more widely supported
                // Responses API is only for specific models that explicitly support it
                Arc::new(ChatCompletionsDriver::new(settings.clone()))
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
    #[allow(dead_code)]
    pub fn settings(&self) -> &LlmSettings {
        &self.settings
    }

    /// Get the MCP registry.
    #[must_use]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(clippy::too_many_lines)]
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send> {
        let request_id = Uuid::new_v4().to_string();
        let tools = self.mcp.openai_tools_json();

        tracing::info!(
            request_id = %request_id,
            message_count = messages.len(),
            tool_count = tools.len(),
            "Starting orchestrator chat"
        );

        // Log initial message history
        for (idx, msg) in messages.iter().enumerate() {
            tracing::debug!(
                request_id = %request_id,
                message_index = idx,
                role = ?msg.role,
                content_length = msg.content.len(),
                has_tool_calls = msg.tool_calls.is_some(),
                "Initial message"
            );
        }

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

            tracing::debug!(
                request_id = %request_id,
                "Converted messages to JSON for driver"
            );

            let mut iteration = 0;

            loop {
                if iteration >= MAX_TOOL_ITERATIONS {
                    tracing::error!(
                        request_id = %request_id,
                        iteration = iteration,
                        max_iterations = MAX_TOOL_ITERATIONS,
                        "Maximum tool loop iterations exceeded"
                    );
                    yield NormalizedEvent::Error {
                        message: "Maximum tool loop iterations exceeded".to_string(),
                        code: Some("MAX_ITERATIONS".to_string()),
                    };
                    break;
                }
                iteration += 1;

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    message_count = message_json.len(),
                    "Starting tool loop iteration"
                );

                let req = LlmRequest {
                    messages: message_json.clone(),
                    tools: tools.clone(),
                };

                // Log the full request being sent to the LLM
                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    messages = ?req.messages,
                    tool_count = req.tools.len(),
                    "Sending request to LLM driver"
                );

                // Stream from the driver
                let driver_stream = match orchestrator.driver.stream(req).await {
                    Ok(s) => {
                        tracing::debug!(
                            request_id = %request_id,
                            iteration = iteration,
                            "Driver stream created successfully"
                        );
                        s
                    }
                    Err(e) => {
                        tracing::error!(
                            request_id = %request_id,
                            iteration = iteration,
                            error = %e,
                            "Failed to create driver stream"
                        );
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
                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        has_tool_calls = has_tool_calls,
                        finish_reason = ?finish_reason,
                        "No tool calls to process, completing stream"
                    );
                    yield NormalizedEvent::Done;
                    break;
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    accumulator_count = tool_accumulators.len(),
                    "Building tool calls from accumulators"
                );

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
                    tracing::warn!(
                        request_id = %request_id,
                        iteration = iteration,
                        "No valid tool calls built from accumulators"
                    );
                    yield NormalizedEvent::Done;
                    break;
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    tool_call_count = tool_calls.len(),
                    "Built tool calls, adding to message history"
                );

                // Log each tool call
                for (idx, tc) in tool_calls.iter().enumerate() {
                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tc.id,
                        tool_name = %tc.function.name,
                        args_length = tc.function.arguments.len(),
                        "Tool call to execute"
                    );
                    tracing::debug!(
                        request_id = %request_id,
                        tool_id = %tc.id,
                        arguments = %tc.function.arguments,
                        "Tool call arguments"
                    );
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

                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    "Added assistant message with tool calls to history"
                );

                // Execute each tool call and emit results
                for (idx, tool_call) in tool_calls.iter().enumerate() {
                    let tool_name = &tool_call.function.name;
                    let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tool_call.id,
                        tool_name = %tool_name,
                        "Executing tool call"
                    );

                    let (content, success) = match orchestrator.mcp.call_namespaced_tool(tool_name, arguments.clone()).await {
                        Ok(result) => {
                            let content = serde_json::to_string(&result).unwrap_or_default();
                            tracing::info!(
                                request_id = %request_id,
                                iteration = iteration,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                result_length = content.len(),
                                "Tool call succeeded"
                            );
                            tracing::debug!(
                                request_id = %request_id,
                                tool_id = %tool_call.id,
                                result = %content,
                                "Tool call result"
                            );
                            (content, true)
                        }
                        Err(e) => {
                            let error_msg = format!("Error: {e}");
                            tracing::error!(
                                request_id = %request_id,
                                iteration = iteration,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                error = %e,
                                "Tool call failed"
                            );
                            (error_msg, false)
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

                    tracing::debug!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_id = %tool_call.id,
                        "Added tool result to message history"
                    );
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    "All tool calls executed, continuing to next iteration"
                );

                // Continue the loop to get the next response
            }
        };

        Ok(stream)
    }

    /// Non-streaming chat for simple requests (e.g., title generation).
    /// 
    /// This collects all message deltas into a single string response.
    pub async fn chat_non_streaming(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let request_id = Uuid::new_v4().to_string();
        let tools = Vec::new(); // No tools for simple requests

        tracing::debug!(
            request_id = %request_id,
            message_count = messages.len(),
            "Starting non-streaming chat"
        );

        let message_json: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect();

        let req = LlmRequest {
            messages: message_json,
            tools,
        };

        // Stream from the driver and collect message deltas
        let mut stream = self.driver.stream(req).await?;
        let mut content = String::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(NormalizedEvent::MessageDelta { text }) => {
                    content.push_str(&text);
                }
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = %e, "Error in stream");
                    return Err(e);
                }
                _ => {} // Ignore other events
            }
        }

        tracing::debug!(
            request_id = %request_id,
            content_length = content.len(),
            "Non-streaming chat completed"
        );

        Ok(content)
    }
}
