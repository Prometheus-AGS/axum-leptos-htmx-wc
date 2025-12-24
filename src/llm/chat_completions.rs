//! `OpenAI` Chat Completions API driver.
//!
//! This module implements the [`LlmDriver`] trait for the `OpenAI` Chat Completions
//! API (`/v1/chat/completions`), supporting streaming responses and tool calls.

use std::collections::BTreeMap;

use futures::{Stream, StreamExt};

use crate::normalized::NormalizedEvent;

use super::{LlmDriver, LlmRequest, LlmSettings};

/// Accumulated state for a streaming tool call.
#[derive(Default)]
struct ToolAccum {
    id: Option<String>,
    name: Option<String>,
    args: String,
}

/// Driver for the `OpenAI` Chat Completions API.
///
/// Connects to `/v1/chat/completions` and streams responses as
/// [`NormalizedEvent`]s.
#[derive(Clone)]
pub struct ChatCompletionsDriver {
    http: reqwest::Client,
    settings: LlmSettings,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for ChatCompletionsDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatCompletionsDriver")
            .field("settings", &self.settings)
            .finish()
    }
}

impl ChatCompletionsDriver {
    /// Create a new Chat Completions driver with the given settings.
    #[must_use]
    pub fn new(settings: LlmSettings) -> Self {
        Self {
            http: reqwest::Client::new(),
            settings,
        }
    }
}

#[async_trait::async_trait]
impl LlmDriver for ChatCompletionsDriver {
    #[allow(clippy::too_many_lines)]
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
    {
        // Build URL based on provider
        let url = self
            .settings
            .provider
            .build_chat_url(&self.settings.base_url, &self.settings.model);

        tracing::info!(
            url = %url,
            model = %self.settings.model,
            provider = ?self.settings.provider,
            message_count = req.messages.len(),
            tool_count = req.tools.len(),
            "Chat Completions: Starting stream request"
        );

        // Build request body
        let mut body = serde_json::json!({
            "model": self.settings.model,
            "stream": true,
            "stream_options": {
                "include_usage": true
            },
            "messages": req.messages,
            "tools": if req.tools.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::Array(req.tools)
            }
        });

        // Add parallel_tool_calls if specified and supported
        // Note: GPT-5.x models don't support parallel_tool_calls parameter
        let is_gpt5_model = self.settings.model.starts_with("gpt-5");

        if let Some(parallel) = self.settings.parallel_tool_calls {
            if is_gpt5_model {
                tracing::debug!(
                    model = %self.settings.model,
                    "Skipping parallel_tool_calls for GPT-5.x model (not supported)"
                );
            } else if self.settings.provider.supports_parallel_tools() {
                body["parallel_tool_calls"] = serde_json::json!(parallel);
                tracing::debug!(
                    parallel_tool_calls = parallel,
                    "Added parallel_tool_calls to request"
                );
            } else {
                tracing::debug!(
                    provider = ?self.settings.provider,
                    "Provider does not support parallel_tool_calls"
                );
            }
        }

        // Log the full request body
        tracing::debug!(
            request_body = %serde_json::to_string_pretty(&body).unwrap_or_default(),
            "Chat Completions: Full request body"
        );

        let mut rb = self.http.post(&url).json(&body);

        // Add authentication header
        if let Some(k) = &self.settings.api_key {
            rb = rb.bearer_auth(k);
            tracing::trace!("Added bearer auth to request");
        }

        tracing::debug!("Sending HTTP request to LLM API");
        let resp = rb.send().await?;

        let status = resp.status();
        tracing::info!(
            status = %status,
            "Received response from LLM API"
        );

        // Check for error status and parse error details if present
        if !status.is_success() {
            let error_body = resp
                .text()
                .await
                .unwrap_or_else(|_| String::from("Failed to read error body"));

            // Try to parse as JSON to extract detailed error information
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_body) {
                let error_obj = &error_json["error"];
                let error_message = error_obj["message"].as_str().unwrap_or("Unknown error");
                let error_type = error_obj["type"].as_str().unwrap_or("unknown");
                let error_param = error_obj["param"].as_str();
                let error_code = error_obj["code"].as_str();

                tracing::error!(
                    status = %status,
                    error_type = error_type,
                    error_message = error_message,
                    error_param = ?error_param,
                    error_code = ?error_code,
                    full_error_body = %error_body,
                    "LLM API returned error with details"
                );

                // Create a detailed error message
                let mut detailed_error = format!("LLM API error ({status}): {error_message}");
                if let Some(param) = error_param {
                    detailed_error.push_str(" [parameter: ");
                    detailed_error.push_str(param);
                    detailed_error.push(']');
                }
                if let Some(code) = error_code {
                    detailed_error.push_str(" [code: ");
                    detailed_error.push_str(code);
                    detailed_error.push(']');
                }

                return Err(anyhow::anyhow!(detailed_error));
            }
            // Not JSON, log raw body
            tracing::error!(
                status = %status,
                error_body = %error_body,
                "LLM API returned non-JSON error"
            );
            return Err(anyhow::anyhow!("LLM API error ({status}): {error_body}"));
        }

        let byte_stream = resp.bytes_stream();

        tracing::debug!("Starting to process response stream");

        let out = async_stream::try_stream! {
            let mut buf = Vec::<u8>::new();
            let mut tool_accum: BTreeMap<usize, ToolAccum> = BTreeMap::new();
            let mut chunk_count = 0;
            let mut event_count = 0;

            futures::pin_mut!(byte_stream);
            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk?;
                chunk_count += 1;
                buf.extend_from_slice(&chunk);

                tracing::trace!(
                    chunk_number = chunk_count,
                    chunk_size = chunk.len(),
                    buffer_size = buf.len(),
                    "Received chunk from stream"
                );

                while let Some(pos) = find_double_newline(&buf) {
                    let frame = buf.drain(..pos + 2).collect::<Vec<_>>();
                    let text = String::from_utf8_lossy(&frame);

                    for line in text.lines() {
                        let line = line.trim();
                        if !line.starts_with("data:") {
                            continue;
                        }
                        let data = line.trim_start_matches("data:").trim();

                        if data == "[DONE]" {
                            tracing::info!(
                                chunk_count = chunk_count,
                                event_count = event_count,
                                "Received [DONE] signal from API"
                            );
                            yield NormalizedEvent::Done;
                            continue;
                        }

                        tracing::trace!(
                            event_number = event_count,
                            data_length = data.len(),
                            "Processing SSE event"
                        );

                        let v: serde_json::Value = serde_json::from_str(data)?;

                        // Check for usage information (sent in final chunk)
                        if let Some(usage) = v.get("usage")
                            && let (Some(prompt), Some(completion), Some(total)) = (
                                usage
                                    .get("prompt_tokens")
                                    .and_then(serde_json::Value::as_u64),
                                usage
                                    .get("completion_tokens")
                                    .and_then(serde_json::Value::as_u64),
                                usage
                                    .get("total_tokens")
                                    .and_then(serde_json::Value::as_u64),
                            )
                        {
                            event_count += 1;
                            tracing::info!(
                                prompt_tokens = prompt,
                                completion_tokens = completion,
                                total_tokens = total,
                                "Received usage information from API"
                            );
                            #[allow(clippy::cast_possible_truncation)]
                            yield NormalizedEvent::Usage {
                                prompt_tokens: prompt as u32,
                                completion_tokens: completion as u32,
                                total_tokens: total as u32,
                            };
                        }

                        let choice = &v["choices"][0];
                        let delta = &choice["delta"];

                        // Assistant text delta
                        if let Some(s) = delta.get("content").and_then(|x| x.as_str())
                            && !s.is_empty() {
                                event_count += 1;
                                tracing::trace!(
                                    event_number = event_count,
                                    delta_length = s.len(),
                                    "Emitting message delta"
                                );
                                yield NormalizedEvent::MessageDelta { text: s.to_string() };
                            }

                        // Tool calls streaming deltas
                        if let Some(arr) = delta.get("tool_calls").and_then(|x| x.as_array()) {
                            for tc in arr {
                                let idx = tc.get("index").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
                                let id = tc.get("id").and_then(|x| x.as_str()).map(ToString::to_string);
                                let name = tc.get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|x| x.as_str())
                                    .map(ToString::to_string);
                                let args_delta = tc.get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|x| x.as_str())
                                    .map(ToString::to_string);

                                tracing::debug!(
                                    call_index = idx,
                                    id = ?id,
                                    name = ?name,
                                    has_args_delta = args_delta.is_some(),
                                    "Processing tool call delta"
                                );

                                let entry = tool_accum.entry(idx).or_default();
                                if entry.id.is_none() {
                                    entry.id.clone_from(&id);
                                }
                                if entry.name.is_none() {
                                    entry.name.clone_from(&name);
                                }
                                if let Some(ad) = &args_delta {
                                    entry.args.push_str(ad);
                                }

                                event_count += 1;
                                yield NormalizedEvent::ToolCallDelta {
                                    call_index: idx,
                                    id,
                                    name,
                                    arguments_delta: args_delta,
                                };
                            }
                        }

                        // Completion boundary: signal tool phase via finish_reason
                        if let Some(fr) = choice.get("finish_reason").and_then(|x| x.as_str()) {
                            tracing::info!(
                                finish_reason = %fr,
                                tool_accum_count = tool_accum.len(),
                                "Received finish_reason from API"
                            );

                            if fr == "tool_calls" {
                                tracing::info!(
                                    tool_count = tool_accum.len(),
                                    "Emitting complete tool calls"
                                );

                                // Emit complete tool calls we've assembled
                                for (idx, a) in &tool_accum {
                                    if let (Some(id), Some(name)) = (&a.id, &a.name) {
                                        tracing::info!(
                                            call_index = idx,
                                            id = %id,
                                            name = %name,
                                            args_length = a.args.len(),
                                            "Emitting ToolCallComplete"
                                        );
                                        tracing::debug!(
                                            call_index = idx,
                                            id = %id,
                                            arguments = %a.args,
                                            "Complete tool call arguments"
                                        );

                                        event_count += 1;
                                        yield NormalizedEvent::ToolCallComplete {
                                            call_index: *idx,
                                            id: id.clone(),
                                            name: name.clone(),
                                            arguments_json: a.args.clone(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tracing::info!(
                total_chunks = chunk_count,
                total_events = event_count,
                "Stream processing complete"
            );
        };

        Ok(Box::pin(out))
    }
}

/// Find the position of a double newline in the buffer.
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}
