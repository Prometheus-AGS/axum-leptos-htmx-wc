# Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          Browser                                 │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    HTML Form                                │ │
│  │  - HTMX: hx-post="/api/chat"                               │ │
│  │  - Extension: hx-ext="json-enc"                            │ │
│  │  - Auto-converts form data → JSON                          │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            │ POST (application/json)              │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                  HTMX Response Handler                      │ │
│  │  - Receives JSON response                                   │ │
│  │  - Calls: chat-stream.startStream(json)                    │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              <chat-stream> Web Component                    │ │
│  │                                                              │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │           SSEConnection (EventSource)                 │  │ │
│  │  │  - Connects to stream_url                             │  │ │
│  │  │  - Listens for SSE events                             │  │ │
│  │  │  - Parses JSON payloads                               │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  │                            │                                 │ │
│  │                            ▼                                 │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │              Event Handler (handleEvent)              │  │ │
│  │  │  • stream.start     → Set request ID                  │  │ │
│  │  │  • message.delta    → Append text                     │  │ │
│  │  │  • thinking.delta   → Update thinking panel           │  │ │
│  │  │  • reasoning.delta  → Update reasoning panel          │  │ │
│  │  │  • citation.added   → Add citation                    │  │ │
│  │  │  • memory.update    → Log to console                  │  │ │
│  │  │  • tool_call.delta  → Stream tool call                │  │ │
│  │  │  • tool_call.complete → Finalize tool call            │  │ │
│  │  │  • tool_result      → Display result                  │  │ │
│  │  │  • error            → Show error                      │  │ │
│  │  │  • done             → Finalize display                │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  │                            │                                 │ │
│  │                            ▼                                 │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │           Render Methods (renderTranscript)           │  │ │
│  │  │  - renderMessage()    → Message bubbles               │  │ │
│  │  │  - renderThinking()   → Collapsible panel             │  │ │
│  │  │  - renderReasoning()  → Collapsible panel             │  │ │
│  │  │  - renderToolCall()   → Tool card with JSON           │  │ │
│  │  │  - renderToolResult() → Result card with JSON         │  │ │
│  │  │  - renderCitations()  → Source links                  │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                             ▲
                             │ GET /api/chat/stream?session_id=...
                             │ (text/event-stream)
                             │
┌─────────────────────────────────────────────────────────────────┐
│                      Axum Server (Rust)                          │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              POST /api/chat                                 │ │
│  │  Input:  Json<ChatRequest>                                  │ │
│  │  Output: Json<ChatResponse>                                 │ │
│  │    { session_id, stream_url }                               │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              GET /api/chat/stream                           │ │
│  │  Input:  Query<StreamQuery>                                 │ │
│  │  Output: SSE Stream (Body::from_stream)                     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Orchestrator                             │ │
│  │  - chat_with_history(messages)                              │ │
│  │  - Manages LLM driver and MCP tools                         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                   LLM Driver                                │ │
│  │  - ChatCompletionsDriver (OpenAI API)                       │ │
│  │  - ResponsesDriver (Responses API)                          │ │
│  │  - Streams raw LLM events                                   │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              NormalizedEvent Conversion                     │ │
│  │  - Converts LLM-specific events → NormalizedEvent enum      │ │
│  │  - Handles tool execution                                   │ │
│  │  - Formats SSE output                                       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                      │
│                            ▼                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                  SSE Event Formatter                        │ │
│  │  sse_event(NormalizedEvent) → String                        │ │
│  │    event: message.delta                                     │ │
│  │    data: {"type":"message.delta","data":{"text":"..."}}    │ │
│  │                                                              │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Event Flow Example

### Simple Message

```
User types "Hello" and presses Enter

1. HTMX json-enc extension converts form to JSON:
   POST /api/chat
   Content-Type: application/json
   {"message": "Hello"}

2. Rust handler creates session:
   Response: {"session_id": "abc123", "stream_url": "/api/chat/stream?session_id=abc123"}

3. HTMX calls: chat-stream.startStream(response)

4. Web Component connects to SSE stream:
   GET /api/chat/stream?session_id=abc123

5. Rust streams events:
   event: stream.start
   data: {"type":"stream.start","data":{"request_id":"xyz"}}

   event: message.delta
   data: {"type":"message.delta","data":{"text":"Hi"}}

   event: message.delta
   data: {"type":"message.delta","data":{"text":" there"}}

   event: message.delta
   data: {"type":"message.delta","data":{"text":"!"}}

   event: done
   data: {"type":"done"}

6. Web Component renders:
   - Creates assistant message bubble
   - Appends "Hi" → "Hi there" → "Hi there!"
   - Finalizes on done event
```

### Tool Call Example

```
User types "What time is it?"

1-4. Same as above

5. Rust streams events:
   event: stream.start
   data: {"type":"stream.start","data":{"request_id":"xyz"}}

   event: tool_call.delta
   data: {"type":"tool_call.delta","data":{"call_index":0,"name":"time::current_time"}}

   event: tool_call.delta
   data: {"type":"tool_call.delta","data":{"call_index":0,"arguments_delta":"{"}}

   event: tool_call.delta
   data: {"type":"tool_call.delta","data":{"call_index":0,"arguments_delta":"\"timezone\""}}

   event: tool_call.complete
   data: {"type":"tool_call.complete","data":{"call_index":0,"id":"call_123","name":"time::current_time","arguments_json":"{\"timezone\":\"UTC\"}"}}

   event: tool_result
   data: {"type":"tool_result","data":{"id":"call_123","name":"time::current_time","content":"2024-01-15T10:30:00Z","success":true}}

   event: message.delta
   data: {"type":"message.delta","data":{"text":"It's"}}

   event: message.delta
   data: {"type":"message.delta","data":{"text":" 10:30 AM UTC"}}

   event: done
   data: {"type":"done"}

6. Web Component renders:
   - Tool call card (streaming → complete)
   - Tool result card with formatted JSON
   - Assistant message with response
```

## Key Design Principles

### 1. Pure JSON/SSE
- No mixed protocols
- Clear separation of concerns
- Standard HTTP/SSE semantics

### 2. Type Safety
- Rust: `NormalizedEvent` enum with serde
- TypeScript: `NormalizedEvent` interface
- Compile-time guarantees

### 3. Extensibility
- New event types: Add to enum + handler
- New display: Add render method
- New LLM: Implement driver trait

### 4. Progressive Enhancement
- Works without JavaScript (form submission)
- Enhanced with HTMX (no page reload)
- Enhanced with Web Components (streaming UI)

### 5. Declarative UI
- HTMX attributes define behavior
- No manual fetch/XHR code
- Web Components encapsulate complexity

## Technology Stack

### Backend
- **Axum**: Web framework
- **Tokio**: Async runtime
- **Serde**: JSON serialization
- **async_stream**: SSE streaming

### Frontend
- **HTMX**: Declarative AJAX
- **Alpine.js**: Reactive attributes
- **Web Components**: Custom elements
- **TypeScript**: Type safety

### Communication
- **JSON**: Request/response format
- **SSE**: Server-to-client streaming
- **EventSource**: Browser SSE client
