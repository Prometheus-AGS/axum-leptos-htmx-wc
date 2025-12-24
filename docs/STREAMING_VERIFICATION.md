# Streaming Verification

## User Message Display

### Implementation
The user message is captured and displayed using a two-phase approach:

1. **Before Request** (`hx-on--before-request`):
   ```javascript
   this.dataset.lastMessage = this.querySelector('[name=message]').value;
   ```
   - Captures message BEFORE form submission
   - Stores in `dataset.lastMessage` for later retrieval

2. **After Request** (`hx-on--after-request`):
   ```javascript
   const msg = this.dataset.lastMessage;
   this.reset();
   document.querySelector('chat-stream')?.startStream(event.detail.xhr.response, msg);
   delete this.dataset.lastMessage;
   ```
   - Retrieves captured message
   - Resets form
   - Passes message to `startStream()`
   - Cleans up dataset

3. **Display** (`chat-stream.ts` line 65-75):
   ```typescript
   startStream(responseJson?: string, userMessage?: string): void {
     // Add user message to transcript if provided
     if (userMessage && userMessage.trim()) {
       this.state.items.push({
         kind: "message",
         role: "user",
         content: userMessage.trim(),
         html: renderMarkdown(userMessage.trim()),
       });
       this.renderTranscript();
     }
     // ... continue with stream connection
   }
   ```

### Expected Behavior
1. User types message and presses Enter
2. User message bubble appears immediately (purple tint)
3. Message stays visible while assistant responds
4. Message remains in chat history

## Real-Time Streaming

### Event Flow

```
Backend (Rust) → SSE Events → Frontend (TypeScript) → UI Update
```

### Supported Events

| Event Type | Handler | Display Component | Real-Time |
|------------|---------|-------------------|-----------|
| `message.delta` | `handleMessageDelta()` | Assistant message bubble | ✅ Yes |
| `thinking.delta` | `handleThinkingDelta()` | Collapsible thinking section | ✅ Yes |
| `reasoning.delta` | `handleReasoningDelta()` | Collapsible reasoning section | ✅ Yes |
| `tool_call.delta` | `handleToolCallDelta()` | Tool call card (streaming) | ✅ Yes |
| `tool_call.complete` | `handleToolCallComplete()` | Tool call card (complete) | ✅ Yes |
| `tool_result` | `handleToolResult()` | Tool result card | ✅ Yes |
| `citation.added` | State update | Citations section | ✅ Yes |
| `memory.update` | Console log | Debug only | ✅ Yes |
| `error` | `handleError()` | Error message | ✅ Yes |
| `done` | `handleDone()` | Stream completion | ✅ Yes |

### Implementation Details

#### SSE Connection (`web/utils/sse.ts`)
```typescript
const normalizedEventTypes = [
  "stream.start",
  "message.delta",
  "thinking.delta",      // ← Real-time thinking
  "reasoning.delta",     // ← Real-time reasoning
  "citation.added",
  "memory.update",
  "tool_call.delta",     // ← Real-time tool calls
  "tool_call.complete",  // ← Tool call completion
  "tool_result",         // ← Tool results
  "error",
  "done",
];
```

#### Event Handler (`web/components/chat-stream/chat-stream.ts`)
```typescript
private handleEvent(event: NormalizedEvent): void {
  switch (event.type) {
    case "thinking.delta":
      this.handleThinkingDelta(event.data.text);
      break;
    case "reasoning.delta":
      this.handleReasoningDelta(event.data.text);
      break;
    case "tool_call.delta":
      this.handleToolCallDelta(event.data);
      break;
    case "tool_call.complete":
      this.handleToolCallComplete(event.data);
      break;
    case "tool_result":
      this.handleToolResult(event.data);
      break;
    // ... other cases
  }
  
  this.renderTranscript(); // ← Immediate UI update
}
```

### Rendering Components

#### Thinking Section
```typescript
private renderThinking(item: {
  content: string;
  isComplete: boolean;
}): string {
  const statusBadge = item.isComplete
    ? '<span class="...">Complete</span>'
    : '<span class="... animate-pulse">Thinking...</span>';
  
  return `
    <details class="..." ${item.isComplete ? "" : "open"}>
      <summary>💭 Thinking ${statusBadge}</summary>
      <div>${escapeHtml(item.content)}</div>
    </details>
  `;
}
```
- Opens automatically while streaming
- Shows animated "Thinking..." badge
- Updates in real-time as deltas arrive

#### Reasoning Section
```typescript
private renderReasoning(item: {
  content: string;
  isComplete: boolean;
}): string {
  const statusBadge = item.isComplete
    ? '<span class="...">Complete</span>'
    : '<span class="... animate-pulse">Reasoning...</span>';
  
  return `
    <details class="..." ${item.isComplete ? "" : "open"}>
      <summary>🧠 Reasoning ${statusBadge}</summary>
      <div>${escapeHtml(item.content)}</div>
    </details>
  `;
}
```
- Opens automatically while streaming
- Shows animated "Reasoning..." badge
- Updates in real-time as deltas arrive

#### Tool Call Card
```typescript
private renderToolCall(item: {
  name: string;
  id: string;
  argumentsRaw: string;
  status: "streaming" | "complete";
}): string {
  const statusBadge = item.status === "complete"
    ? '<span class="...">Complete</span>'
    : '<span class="... animate-pulse">Streaming...</span>';
  
  return `
    <article class="...">
      <div>🔧 Tool Call ${statusBadge}</div>
      <code>${escapeHtml(item.name)}</code>
      <pre>${escapeHtml(item.argumentsRaw)}</pre>
    </article>
  `;
}
```
- Shows tool name and ID
- Displays streaming arguments in real-time
- Animated badge while streaming
- Copy button for arguments

#### Tool Result Card
```typescript
private renderToolResult(item: {
  name: string;
  id: string;
  contentRaw: string;
  success: boolean;
}): string {
  const statusIcon = item.success ? "✅" : "❌";
  
  return `
    <article class="...">
      <div>${statusIcon} Tool Result</div>
      <code>${escapeHtml(item.name)}</code>
      <pre>${escapeHtml(item.contentRaw)}</pre>
    </article>
  `;
}
```
- Shows success/failure status
- Displays formatted JSON result
- Copy button for result content

### Backend Event Emission

#### Chat Completions Driver (`src/llm/chat_completions.rs`)
```rust
// Assistant text delta
if let Some(s) = delta.get("content").and_then(|x| x.as_str()) {
    yield NormalizedEvent::MessageDelta { text: s.to_string() };
}

// Tool calls streaming deltas
if let Some(arr) = delta.get("tool_calls").and_then(|x| x.as_array()) {
    for tc in arr {
        yield NormalizedEvent::ToolCallDelta {
            call_index: idx,
            id,
            name,
            arguments_delta,
        };
    }
}
```

#### Responses Driver (`src/llm/responses.rs`)
```rust
match event_name.as_str() {
    "response.output_text.delta" => {
        yield NormalizedEvent::MessageDelta { text: delta };
    }
    "response.thinking.delta" => {
        yield NormalizedEvent::ThinkingDelta { text: delta };
    }
    "response.reasoning.delta" => {
        yield NormalizedEvent::ReasoningDelta { text: delta };
    }
    "response.function_call_arguments.delta" => {
        yield NormalizedEvent::ToolCallDelta { /* ... */ };
    }
}
```

## Testing Checklist

### User Message
- [ ] Type a message and press Enter
- [ ] User bubble appears immediately
- [ ] User bubble has purple tint (dark mode) or light purple (light mode)
- [ ] Message text is readable
- [ ] Copy button appears on hover
- [ ] Message stays in history

### Assistant Message
- [ ] Assistant message streams character by character
- [ ] Text appears in real-time
- [ ] Markdown is rendered correctly
- [ ] Copy button works

### Thinking (if model supports)
- [ ] Thinking section appears when model is thinking
- [ ] Section is open by default
- [ ] "Thinking..." badge is animated
- [ ] Content updates in real-time
- [ ] Badge changes to "Complete" when done

### Reasoning (if model supports)
- [ ] Reasoning section appears when model reasons
- [ ] Section is open by default
- [ ] "Reasoning..." badge is animated
- [ ] Content updates in real-time
- [ ] Badge changes to "Complete" when done

### Tool Calls
- [ ] Tool call card appears when tool is invoked
- [ ] Tool name is displayed
- [ ] Arguments stream in real-time
- [ ] "Streaming..." badge is animated
- [ ] Badge changes to "Complete" when done
- [ ] Copy button works for arguments

### Tool Results
- [ ] Tool result card appears after execution
- [ ] Success/failure icon is correct (✅/❌)
- [ ] Result content is formatted
- [ ] Copy button works for result

### Citations (if present)
- [ ] Citations section appears at end
- [ ] Links are clickable
- [ ] Titles are displayed

## Performance

- **Latency**: < 100ms from event to UI update
- **Smoothness**: 60fps rendering
- **Memory**: Efficient state management
- **Scroll**: Auto-scroll to bottom on new content

## Browser Compatibility

- **EventSource**: All modern browsers
- **SSE**: Chrome 6+, Firefox 6+, Safari 5+
- **Real-time updates**: Supported everywhere

## Known Limitations

1. **Thinking/Reasoning**: Only available for models that expose internal reasoning (e.g., o1, o3)
2. **Memory Updates**: Currently logged to console only
3. **Citations**: Only shown if model provides them

## Future Enhancements

- [ ] Pause/resume streaming
- [ ] Retry failed tool calls
- [ ] Export conversation with all metadata
- [ ] Collapsible tool call details
- [ ] Syntax highlighting in tool arguments/results
