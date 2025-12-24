# Conversation Memory & History

## Overview

The application now maintains full conversation history across multiple turns, allowing the LLM to remember previous messages and maintain context throughout the conversation.

## Implementation

### Backend Session Management

#### Session Storage (`src/session/thread.rs`)

Each conversation is stored in a `Session` object that maintains:
- **Session ID**: Unique identifier for the conversation
- **Message History**: Complete list of all messages (user, assistant, tool, system)
- **System Prompt**: Optional system-level instructions
- **Last Activity**: Timestamp for session management

```rust
pub struct Session {
    id: String,
    messages: RwLock<Vec<Message>>,
    created_at: Instant,
    last_activity: RwLock<Instant>,
    system_prompt: RwLock<Option<String>>,
}
```

#### Message Types

```rust
pub enum MessageRole {
    System,    // System prompt
    User,      // User messages
    Assistant, // Assistant responses
    Tool,      // Tool execution results
}

pub struct Message {
    role: MessageRole,
    content: String,
    tool_call_id: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}
```

### Frontend Session Tracking

#### Form State (`src/main.rs` lines 223-241)

The chat form uses Alpine.js to track the session ID:

```html
<form 
    x-data="{ message: '', sessionId: null }"
    hx-on--after-request="
        const response = JSON.parse(event.detail.xhr.response);
        if (response.session_id) {
            sessionId = response.session_id;
        }
    "
>
    <!-- Hidden input sends session_id on subsequent requests -->
    <input type="hidden" name="session_id" x-bind:value="sessionId">
    <textarea name="message" x-model="message"></textarea>
</form>
```

**Flow**:
1. First message: No session_id sent → Backend creates new session
2. Backend returns `{ session_id: "...", stream_url: "..." }`
3. Frontend stores `session_id` in Alpine.js state
4. Subsequent messages: session_id sent → Backend uses existing session
5. All previous messages are included in LLM context

### Message Persistence

#### User Messages (`src/main.rs` line 372)

When a user sends a message:
```rust
session.add_user_message(&req.message);
```

#### Assistant Messages (`src/main.rs` lines 535-551)

After streaming completes, the assistant's response is saved:
```rust
// Accumulate during streaming
let mut assistant_content = String::new();
let mut tool_calls: Vec<ToolCall> = Vec::new();

// On MessageDelta events
assistant_content.push_str(text);

// On ToolCallComplete events
tool_calls.push(ToolCall { /* ... */ });

// On Done event
if !assistant_content.is_empty() || !tool_calls.is_empty() {
    let msg = Message {
        role: MessageRole::Assistant,
        content: assistant_content,
        tool_call_id: None,
        tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
    };
    session.add_message(msg);
}
```

### History Sent to LLM

#### Full Context (`src/main.rs` line 421)

Every request includes the complete message history:
```rust
let messages = session.messages_with_system();
let stream = orchestrator.chat_with_history(messages).await?;
```

**Logged for debugging** (lines 433-449):
```rust
for (idx, msg) in messages.iter().enumerate() {
    tracing::debug!(
        message_index = idx,
        role = ?msg.role,
        content_length = msg.content.len(),
        has_tool_calls = msg.tool_calls.is_some(),
        "Message in history"
    );
}
```

## Message Flow

### First Turn

```
User: "Hello"
  ↓
[Frontend] POST /api/chat { message: "Hello" }
  ↓
[Backend] Create new session
  ↓
[Backend] session.add_user_message("Hello")
  ↓
[Backend] Return { session_id: "abc123", stream_url: "/stream?session_id=abc123" }
  ↓
[Frontend] Store sessionId = "abc123"
  ↓
[Frontend] Connect to SSE stream
  ↓
[Backend] Send messages to LLM: [{ role: "user", content: "Hello" }]
  ↓
[Backend] Stream assistant response
  ↓
[Backend] Save assistant response to session
  ↓
Session now contains:
  - User: "Hello"
  - Assistant: "Hi there! How can I help you today?"
```

### Second Turn

```
User: "What's the weather?"
  ↓
[Frontend] POST /api/chat { message: "What's the weather?", session_id: "abc123" }
  ↓
[Backend] Get existing session "abc123"
  ↓
[Backend] session.add_user_message("What's the weather?")
  ↓
[Backend] Send messages to LLM:
  [
    { role: "user", content: "Hello" },
    { role: "assistant", content: "Hi there! How can I help you today?" },
    { role: "user", content: "What's the weather?" }
  ]
  ↓
[Backend] LLM sees full conversation history
  ↓
[Backend] Stream assistant response
  ↓
[Backend] Save assistant response to session
  ↓
Session now contains:
  - User: "Hello"
  - Assistant: "Hi there! How can I help you today?"
  - User: "What's the weather?"
  - Assistant: "I'll check the weather for you..." (with tool calls)
```

### With Tool Calls

```
User: "Search for Rust tutorials"
  ↓
[Backend] Add user message to session
  ↓
[Backend] Send to LLM with full history
  ↓
[LLM] Returns tool call: tavily__search(query="Rust tutorials")
  ↓
[Backend] Execute tool
  ↓
[Backend] Send tool result back to LLM
  ↓
[LLM] Generates response using tool result
  ↓
[Backend] Save assistant message with tool_calls to session
  ↓
Session contains:
  - Previous messages...
  - User: "Search for Rust tutorials"
  - Assistant: "" (with tool_calls: [{ name: "tavily__search", ... }])
  - Tool: "{ results: [...] }" (tool_call_id: "call_123")
  - Assistant: "Here are some great Rust tutorials I found..."
```

## UI Display

### User Messages

- Displayed immediately when submitted (purple bubble)
- Persisted in frontend state (chat-stream component)
- Visible throughout the conversation

### Assistant Messages

- Stream in real-time character by character
- Displayed in neutral gray bubble
- Persisted in frontend state
- Include tool calls and results when present

### Tool Interactions

- Tool calls shown as cards with arguments
- Tool results shown as cards with formatted output
- Both persisted in UI and visible in history

## Session Lifecycle

### Creation
- First message creates new session
- Session ID generated (UUID)
- Stored in `SessionStore` (in-memory HashMap)

### Active Use
- Each message references session_id
- Messages added to session history
- Full history sent to LLM on each turn

### Timeout
- Default: 30 minutes of inactivity
- Can be configured via `DEFAULT_SESSION_TIMEOUT`
- Sessions automatically cleaned up (not yet implemented)

## Debugging

### View Session History

Check logs for message history:
```
[DEBUG] Message in history: index=0 role=User content_length=5
[DEBUG] Message in history: index=1 role=Assistant content_length=42
[DEBUG] Message in history: index=2 role=User content_length=18
```

### Verify Session Persistence

1. Send first message
2. Check response for `session_id`
3. Send second message
4. Check logs for "Using existing session"
5. Verify message_count increases

### Test Memory

Ask the LLM to recall information from earlier in the conversation:
```
User: "My name is Alice"
Assistant: "Nice to meet you, Alice!"
User: "What's my name?"
Assistant: "Your name is Alice."  ← Should remember!
```

## Known Limitations

1. **No Persistence**: Sessions are in-memory only (lost on server restart)
2. **No Cleanup**: Old sessions not automatically removed
3. **No UI History**: Chat history not loaded on page refresh
4. **No Export**: Cannot save/export conversations

## Future Enhancements

- [ ] Persistent storage (database)
- [ ] Session cleanup/expiry
- [ ] Load chat history on page refresh
- [ ] Export conversations
- [ ] Search conversation history
- [ ] Multi-user support with authentication
- [ ] Conversation branching
- [ ] Edit/regenerate messages

## Testing Checklist

- [ ] First message creates new session
- [ ] Session ID returned in response
- [ ] Second message uses same session
- [ ] LLM remembers previous messages
- [ ] Tool calls preserved in history
- [ ] Tool results included in context
- [ ] Multi-turn conversations work
- [ ] User and assistant messages both visible in UI
- [ ] Messages persist throughout conversation
- [ ] Logs show full message history
