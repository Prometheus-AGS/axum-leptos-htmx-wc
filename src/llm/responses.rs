//! `OpenAI` Responses API driver.
//!
//! This module implements the [`LlmDriver`] trait for the `OpenAI` Responses
//! API (`/v1/responses`), supporting streaming responses with rich event types.

use futures::{Stream, StreamExt};

use crate::normalized::NormalizedEvent;

use super::{LlmDriver, LlmRequest, LlmSettings};

/// Driver for the `OpenAI` Responses API.
///
/// Connects to `/v1/responses` and streams responses as [`NormalizedEvent`]s.
/// This driver supports extended event types like thinking and reasoning.
#[derive(Clone)]
pub struct ResponsesDriver {
    http: reqwest::Client,
    settings: LlmSettings,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for ResponsesDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponsesDriver")
            .field("settings", &self.settings)
            .finish()
    }
}

impl ResponsesDriver {
    /// Create a new Responses driver with the given settings.
    #[must_use]
    pub fn new(settings: LlmSettings) -> Self {
        Self {
            http: reqwest::Client::new(),
            settings,
        }
    }
}

#[async_trait::async_trait]
impl LlmDriver for ResponsesDriver {
    #[allow(clippy::too_many_lines)]
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
    {
        let url = format!(
            "{}/v1/responses",
            self.settings.base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.settings.model,
            "stream": true,
            "input": req.messages,
            "tools": if req.tools.is_empty() { serde_json::Value::Null } else { serde_json::Value::Array(req.tools) }
        });

        let mut rb = self.http.post(&url).json(&body);
        if let Some(k) = &self.settings.api_key {
            rb = rb.bearer_auth(k);
        }

        tracing::debug!("Sending HTTP request to Responses API");
        let resp = rb.send().await?;
        
        let status = resp.status();
        tracing::info!(
            status = %status,
            "Received response from Responses API"
        );
        
        // Check for error status and parse error details if present
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_else(|_| String::from("Failed to read error body"));
            
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
                    "Responses API returned error with details"
                );
                
                // Create a detailed error message
                let mut detailed_error = format!("Responses API error ({status}): {error_message}");
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
                "Responses API returned non-JSON error"
            );
            return Err(anyhow::anyhow!("Responses API error ({status}): {error_body}"));
        }
        
        let byte_stream = resp.bytes_stream();

        let out = async_stream::try_stream! {
            let mut buf = Vec::<u8>::new();
            let mut current_event_name: Option<String> = None;

            futures::pin_mut!(byte_stream);
            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk?;
                buf.extend_from_slice(&chunk);

                while let Some(pos) = find_double_newline(&buf) {
                    let frame = buf.drain(..pos + 2).collect::<Vec<_>>();
                    let text = String::from_utf8_lossy(&frame);

                    let mut data_line: Option<String> = None;

                    for line in text.lines() {
                        let line = line.trim();
                        if line.starts_with("event:") {
                            current_event_name = Some(line.trim_start_matches("event:").trim().to_string());
                        } else if line.starts_with("data:") {
                            data_line = Some(line.trim_start_matches("data:").trim().to_string());
                        }
                    }

                    if let Some(d) = data_line {
                        if d == "[DONE]" {
                            yield NormalizedEvent::Done;
                            continue;
                        }

                        let v: serde_json::Value = serde_json::from_str(&d)?;
                        let ev = current_event_name.clone().unwrap_or_default();

                        match ev.as_str() {
                            // Text output delta
                            "response.output_text.delta" => {
                                if let Some(delta) = v.get("delta").and_then(|x| x.as_str())
                                    && !delta.is_empty() {
                                        yield NormalizedEvent::MessageDelta { text: delta.to_string() };
                                    }
                            }

                            // Thinking delta (for models that expose reasoning)
                            "response.thinking.delta" => {
                                if let Some(delta) = v.get("delta").and_then(|x| x.as_str())
                                    && !delta.is_empty() {
                                        yield NormalizedEvent::ThinkingDelta { text: delta.to_string() };
                                    }
                            }

                            // Reasoning delta
                            "response.reasoning.delta" => {
                                if let Some(delta) = v.get("delta").and_then(|x| x.as_str())
                                    && !delta.is_empty() {
                                        yield NormalizedEvent::ReasoningDelta { text: delta.to_string() };
                                    }
                            }

                            // Tool call events
                            "response.function_call_arguments.delta" => {
                                let call_index = v.get("output_index").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
                                let delta = v.get("delta").and_then(|x| x.as_str()).map(ToString::to_string);

                                yield NormalizedEvent::ToolCallDelta {
                                    call_index,
                                    id: None,
                                    name: None,
                                    arguments_delta: delta,
                                };
                            }

                            "response.output_item.added" => {
                                // Check if this is a function call item
                                if let Some(item) = v.get("item")
                                    && item.get("type").and_then(|t| t.as_str()) == Some("function_call") {
                                        let call_index = v.get("output_index").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
                                        let id = item.get("call_id").and_then(|x| x.as_str()).map(ToString::to_string);
                                        let name = item.get("name").and_then(|x| x.as_str()).map(ToString::to_string);

                                        yield NormalizedEvent::ToolCallDelta {
                                            call_index,
                                            id,
                                            name,
                                            arguments_delta: None,
                                        };
                                    }
                            }

                            "response.output_item.done" => {
                                if let Some(item) = v.get("item")
                                    && item.get("type").and_then(|t| t.as_str()) == Some("function_call") {
                                        let call_index = v.get("output_index").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
                                        let id = item.get("call_id").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                                        let name = item.get("name").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                                        let arguments = item.get("arguments").and_then(|x| x.as_str()).unwrap_or("{}").to_string();

                                        yield NormalizedEvent::ToolCallComplete {
                                            call_index,
                                            id,
                                            name,
                                            arguments_json: arguments,
                                        };
                                    }
                            }

                            "response.done" => {
                                yield NormalizedEvent::Done;
                            }

                            // Ignore unknown events
                            _ => {}
                        }
                    }
                }
            }
        };

        Ok(Box::pin(out))
    }
}

/// Find the position of a double newline in the buffer.
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}
