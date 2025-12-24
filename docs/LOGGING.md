# Logging Documentation

This document describes the comprehensive logging added throughout the LLM request pipeline.

## Overview

Extensive logging has been added to all parts of the request pipeline to help debug and monitor:
- API request handling
- LLM orchestration and tool loops
- Chat completions driver
- Tool execution
- Message history and prompts

## Log Levels

The application uses the following log levels:

- **`ERROR`**: Critical errors that prevent operations from completing
- **`WARN`**: Warnings about potentially problematic situations
- **`INFO`**: High-level operational information about request flow
- **`DEBUG`**: Detailed information useful for debugging (includes full arguments, results)
- **`TRACE`**: Very detailed information (includes message deltas, chunk processing)

## Logging by Component

### 1. API Handlers (`src/main.rs`)

#### `api_chat` - POST /api/chat
Logs when a chat request is received and processed:

```rust
INFO  - Received chat request (message, session_id)
DEBUG - Using existing session or Created new session
DEBUG - Added user message to session (message_count)
INFO  - Chat request processed, returning stream URL
```

#### `api_chat_stream` - GET /api/chat/stream
Logs the SSE stream lifecycle and all events:

```rust
INFO  - Starting SSE stream (session_id, has_message)
ERROR - Session not found (if applicable)
DEBUG - Added message from query parameter (if applicable)
INFO  - Starting LLM orchestration (request_id, message_count)
DEBUG - Message in history (for each message: role, content_length, has_tool_calls)
TRACE - Full message content (for each message)
ERROR - Failed to start orchestrator (if error occurs)
DEBUG - Orchestrator stream started

// For each event in the stream:
INFO  - Stream started (request_id)
TRACE - Message delta (delta_length)
DEBUG - Tool call delta (call_index, id, name)
INFO  - Tool call complete (call_index, id, name, args_length)
DEBUG - Tool call arguments (full JSON)
INFO  - Tool result (tool_id, tool_name, success, result_length)
DEBUG - Tool result content (full result)
ERROR - Stream error (error message, code)
INFO  - Stream complete
TRACE - Other event type (for ThinkingDelta, ReasoningDelta, etc.)
```

### 2. Orchestrator (`src/llm/orchestrator.rs`)

#### `chat_with_history`
Logs the complete orchestration process including tool loops:

```rust
INFO  - Starting orchestrator chat (request_id, message_count, tool_count)
DEBUG - Initial message (for each: message_index, role, content_length, has_tool_calls)
DEBUG - Converted messages to JSON for driver

// For each iteration of the tool loop:
ERROR - Maximum tool loop iterations exceeded (if limit reached)
INFO  - Starting tool loop iteration (iteration, message_count)
DEBUG - Sending request to LLM driver (messages, tool_count)
DEBUG - Driver stream created successfully
ERROR - Failed to create driver stream (if error)

// After receiving response:
INFO  - No tool calls to process, completing stream (or continuing)
INFO  - Building tool calls from accumulators (accumulator_count)
WARN  - No valid tool calls built from accumulators (if empty)
INFO  - Built tool calls, adding to message history (tool_call_count)

// For each tool call:
INFO  - Tool call to execute (tool_index, tool_id, tool_name, args_length)
DEBUG - Tool call arguments (full JSON)
INFO  - Executing tool call (tool_index, tool_id, tool_name)
INFO  - Tool call succeeded (tool_id, tool_name, result_length)
DEBUG - Tool call result (full result)
ERROR - Tool call failed (tool_id, tool_name, error)
DEBUG - Added tool result to message history

INFO  - All tool calls executed, continuing to next iteration
```

### 3. Chat Completions Driver (`src/llm/chat_completions.rs`)

#### `stream`
Logs HTTP requests to the LLM API and response processing:

```rust
INFO  - Chat Completions: Starting stream request (url, model, provider, message_count, tool_count)
DEBUG - Added parallel_tool_calls to request (if applicable)
DEBUG - Skipping parallel_tool_calls for GPT-5.x model (not supported) (if GPT-5.x)
DEBUG - Provider does not support parallel_tool_calls (if applicable)
DEBUG - Chat Completions: Full request body (pretty-printed JSON)
TRACE - Added bearer auth to request
DEBUG - Sending HTTP request to LLM API
INFO  - Received response from LLM API (status)

// If error response:
ERROR - LLM API returned error with details (status, error_type, error_message, error_param, error_code, full_error_body)
ERROR - LLM API returned non-JSON error (status, error_body) [if not JSON]

// If success:
DEBUG - Starting to process response stream

// During stream processing:
TRACE - Received chunk from stream (chunk_number, chunk_size, buffer_size)
TRACE - Processing SSE event (event_number, data_length)
INFO  - Received [DONE] signal from API (chunk_count, event_count)
TRACE - Emitting message delta (event_number, delta_length)
DEBUG - Processing tool call delta (call_index, id, name, has_args_delta)
INFO  - Received finish_reason from API (finish_reason, tool_accum_count)
INFO  - Emitting complete tool calls (tool_count)
INFO  - Emitting ToolCallComplete (call_index, id, name, args_length)
DEBUG - Complete tool call arguments (full JSON)
INFO  - Stream processing complete (total_chunks, total_events)
```

### 4. Responses Driver (`src/llm/responses.rs`)

#### `stream`
Logs HTTP requests to the OpenAI Responses API and response processing:

```rust
DEBUG - Sending HTTP request to Responses API
INFO  - Received response from Responses API (status)

// If error response:
ERROR - Responses API returned error with details (status, error_type, error_message, error_param, error_code, full_error_body)
ERROR - Responses API returned non-JSON error (status, error_body) [if not JSON]

// If success:
// (Stream processing logs similar to Chat Completions)
```

**Note:** The Responses API uses a different streaming format (SSE with named events like `response.delta`, `response.tool_call.delta`, etc.) but error handling is identical to the Chat Completions API.

## Viewing Logs

### Development
By default, logs are output to stderr with the `INFO` level. To see more detailed logs:

```bash
# Set log level via environment variable
RUST_LOG=debug cargo run

# Or for even more detail
RUST_LOG=trace cargo run

# Filter to specific modules
RUST_LOG=axum_leptos_htmx_wc::llm=debug cargo run
```

### Production
Configure the log level in your deployment environment:

```bash
RUST_LOG=info ./axum-leptos-htmx-wc
```

## Log Format

Logs use the `tracing` crate and include:
- Timestamp
- Log level
- Module path
- Structured fields (key=value pairs)
- Message

Example:
```
2025-12-23T22:30:45.123456Z  INFO axum_leptos_htmx_wc: Received chat request message="Hello" session_id=None
```

## Debugging Tips

### Tracing a Single Request
1. Find the `request_id` in the logs (generated when SSE stream starts)
2. Filter logs by that request_id:
   ```bash
   cargo run 2>&1 | grep "request_id=abc-123"
   ```

### Debugging Tool Calls
Look for these log sequences:
1. `Tool call to execute` - Shows which tool is being called
2. `Tool call arguments` - Shows the full arguments (DEBUG level)
3. `Tool call succeeded/failed` - Shows the result
4. `Tool result content` - Shows the full result (DEBUG level)

### Debugging LLM Responses
1. Check `Chat Completions: Full request body` for what was sent to the API
2. Check `Received response from LLM API` for HTTP status
3. Check `Received chunk from stream` to see if data is flowing
4. Check `Processing SSE event` to see parsed events

### Debugging LLM API Errors
When the LLM API returns an error (4xx or 5xx status), the system now automatically parses and logs detailed error information:

**Example Error Log:**
```
ERROR axum_leptos_htmx_wc::llm::chat_completions: LLM API returned error with details 
      status=400 Bad Request 
      error_type=invalid_request_error 
      error_message="Unsupported parameter: 'parallel_tool_calls' is not supported with this model." 
      error_param=Some("parallel_tool_calls") 
      error_code=None 
      full_error_body={"error":{"message":"Unsupported parameter: 'parallel_tool_calls' is not supported with this model.","type":"invalid_request_error","param":"parallel_tool_calls","code":null}}
```

**What to look for:**
- **error_message**: The human-readable error description
- **error_param**: Which parameter caused the issue (e.g., `parallel_tool_calls`, `top_p`, `temperature`)
- **error_type**: The category of error (e.g., `invalid_request_error`, `authentication_error`, `rate_limit_error`)
- **error_code**: Optional error code for specific error types
- **full_error_body**: Complete JSON response for detailed debugging

**Common Error Types:**
- `invalid_request_error` - Bad request parameters (check `error_param`)
- `authentication_error` - API key issues
- `rate_limit_error` - Too many requests
- `insufficient_quota` - Billing/quota issues
- `model_not_found` - Invalid model name
- `context_length_exceeded` - Input too long

### Common Issues

#### No logs appearing
- Check `RUST_LOG` environment variable is set
- Ensure you're looking at stderr, not stdout

#### Too many logs
- Use `RUST_LOG=info` to reduce verbosity
- Filter by module: `RUST_LOG=axum_leptos_htmx_wc::llm=info`

#### Missing tool execution logs
- Check if tool calls are being detected (look for "Tool call delta")
- Check for "Maximum tool loop iterations exceeded" error
- Verify tools are registered in MCP (look for "MCP tool discovered")

## Performance Considerations

- `TRACE` level logging can significantly impact performance
- In production, use `INFO` or `WARN` level
- `DEBUG` level is suitable for staging environments
- Structured logging fields are efficient and don't require string formatting unless the log is actually emitted
