# GPT-5.x Model Compatibility

## Issue Summary

When using GPT-5.x models (gpt-5, gpt-5.1, gpt-5.2, etc.) with the OpenAI Chat Completions API, certain parameters that work with GPT-4.x models are **not supported** and will cause a `400 Bad Request` error.

## Root Cause

GPT-5.x models, particularly when using reasoning modes, do not support the following parameters:
- `parallel_tool_calls` - Not supported at all
- `top_p` - Not supported when using reasoning effort
- `temperature` - Not supported when using reasoning effort (except with `reasoning_effort: "none"`)

### Error Example

```
HTTP status client error (400 Bad Request) for url (https://api.openai.com/v1/chat/completions)
```

The actual error message from OpenAI would be:
```json
{
  "error": {
    "message": "Unsupported parameter: 'parallel_tool_calls' is not supported with this model.",
    "type": "invalid_request_error",
    "param": "parallel_tool_calls",
    "code": null
  }
}
```

## Solution

The code has been updated in `src/llm/chat_completions.rs` to detect GPT-5.x models and skip the `parallel_tool_calls` parameter:

```rust
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
        // ...
    }
}
```

## GPT-5.2 Model Variants

GPT-5.2 comes in three variants, each with a different API model identifier:

| Variant | API Model Name | Use Case |
|---------|----------------|----------|
| **GPT-5.2 Thinking** | `gpt-5.2` | Default reasoning model, adaptive computation |
| **GPT-5.2 Instant** | `gpt-5.2-chat-latest` | Fast responses, low latency |
| **GPT-5.2 Pro** | `gpt-5.2-pro` | Maximum quality, highest reasoning effort |

### Model Characteristics

- **Context Window**: 400,000 tokens
- **Max Output Tokens**: 128,000 tokens
- **Knowledge Cutoff**: August 31, 2025
- **Multimodal**: Yes (text, images, audio)
- **Reasoning Modes**: Adaptive (Thinking), None (Instant), Configurable (Pro)

## Configuration

To use GPT-5.2 in your `.env` file:

```bash
LLM_MODEL=gpt-5.2                # For Thinking mode (default)
# OR
LLM_MODEL=gpt-5.2-chat-latest    # For Instant mode (faster)
# OR
LLM_MODEL=gpt-5.2-pro            # For Pro mode (highest quality)
```

## API Endpoints

GPT-5.2 is available in two API formats:

1. **Chat Completions API** (recommended for compatibility):
   ```
   POST https://api.openai.com/v1/chat/completions
   ```

2. **Responses API** (new format with enhanced features):
   ```
   POST https://api.openai.com/v1/responses
   ```

Our implementation uses the Chat Completions API by default for maximum compatibility.

## Debugging

With the comprehensive logging added, you can see the parameter handling in the logs:

```
INFO  axum_leptos_htmx_wc::llm::chat_completions: Chat Completions: Starting stream request 
      url=https://api.openai.com/v1/chat/completions 
      model=gpt-5.2 
      provider=OpenAI 
      message_count=1 
      tool_count=10

DEBUG axum_leptos_htmx_wc::llm::chat_completions: Skipping parallel_tool_calls for GPT-5.x model (not supported) 
      model=gpt-5.2
```

## References

- [OpenAI GPT-5.2 Announcement](https://openai.com/index/introducing-gpt-5-2/)
- [OpenAI API Documentation](https://platform.openai.com/docs/)
- [GPT-5.2 Prompting Guide](https://cookbook.openai.com/examples/gpt-5/gpt-5-2_prompting_guide)
- [Community Discussion on GPT-5.2 Parameters](https://community.openai.com/t/gpt-5-2-not-working-with-reusable-prompts-in-responses-api/1369113)

## Related Files

- `src/llm/chat_completions.rs` - Chat Completions driver with GPT-5.x compatibility
- `src/llm/orchestrator.rs` - LLM orchestration with tool loop
- `docs/LOGGING.md` - Comprehensive logging documentation
- `.env` - LLM configuration (not in git)
