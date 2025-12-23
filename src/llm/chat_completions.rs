//! OpenAI Chat Completions API driver.
//!
//! This module implements the [`LlmDriver`] trait for the OpenAI Chat Completions
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

/// Driver for the OpenAI Chat Completions API.
///
/// Connects to `/v1/chat/completions` and streams responses as
/// [`NormalizedEvent`]s.
#[derive(Clone)]
pub struct ChatCompletionsDriver {
    http: reqwest::Client,
    settings: LlmSettings,
}

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
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
    {
        let url = format!(
            "{}/v1/chat/completions",
            self.settings.base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.settings.model,
            "stream": true,
            "messages": req.messages,
            "tools": if req.tools.is_empty() { serde_json::Value::Null } else { serde_json::Value::Array(req.tools) }
        });

        let mut rb = self.http.post(&url).json(&body);
        if let Some(k) = &self.settings.api_key {
            rb = rb.bearer_auth(k);
        }

        let resp = rb.send().await?.error_for_status()?;
        let byte_stream = resp.bytes_stream();

        let out = async_stream::try_stream! {
            let mut buf = Vec::<u8>::new();
            let mut tool_accum: BTreeMap<usize, ToolAccum> = BTreeMap::new();

            futures::pin_mut!(byte_stream);
            while let Some(chunk) = byte_stream.next().await {
                let chunk = chunk?;
                buf.extend_from_slice(&chunk);

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
                            yield NormalizedEvent::Done;
                            continue;
                        }

                        let v: serde_json::Value = serde_json::from_str(data)?;
                        let choice = &v["choices"][0];
                        let delta = &choice["delta"];

                        // Assistant text delta
                        if let Some(s) = delta.get("content").and_then(|x| x.as_str()) {
                            if !s.is_empty() {
                                yield NormalizedEvent::MessageDelta { text: s.to_string() };
                            }
                        }

                        // Tool calls streaming deltas
                        if let Some(arr) = delta.get("tool_calls").and_then(|x| x.as_array()) {
                            for tc in arr {
                                let idx = tc.get("index").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                                let id = tc.get("id").and_then(|x| x.as_str()).map(ToString::to_string);
                                let name = tc.get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|x| x.as_str())
                                    .map(ToString::to_string);
                                let args_delta = tc.get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|x| x.as_str())
                                    .map(ToString::to_string);

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
                            if fr == "tool_calls" {
                                // Emit complete tool calls we've assembled
                                for (idx, a) in &tool_accum {
                                    if let (Some(id), Some(name)) = (&a.id, &a.name) {
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
        };

        Ok(Box::pin(out))
    }
}

/// Find the position of a double newline in the buffer.
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}
