# State Management Architecture

## Overview

This document describes the client-side state management architecture for the chat application. The system implements a **hybrid state management pattern** that combines Alpine.js global stores with Web Component local state and localStorage persistence.

## Architecture

### Three-Layer State System

```
┌─────────────────────────────────────────────────────┐
│                   Client Side                        │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────────┐      ┌──────────────────┐   │
│  │  Alpine.js Store │◄────►│  localStorage    │   │
│  │  - UI State      │      │  - Conversations │   │
│  │  - Session ID    │      │  - Messages      │   │
│  │  - Active Conv   │      └──────────────────┘   │
│  └────────┬─────────┘                              │
│           │                                         │
│           ▼                                         │
│  ┌──────────────────────────────────────────────┐ │
│  │     Conversation Store (TypeScript)          │ │
│  │     - CRUD operations                        │ │
│  │     - Message history per conversation       │ │
│  │     - Sync with localStorage                 │ │
│  └────────┬─────────────────────────────────────┘ │
│           │                                         │
│           ▼                                         │
│  ┌──────────────────────────────────────────────┐ │
│  │     chat-stream Web Component                │ │
│  │     - Renders messages from store            │ │
│  │     - Handles streaming state                │ │
│  │     - Saves completed messages back to store │ │
│  └──────────────────────────────────────────────┘ │
│                                                      │
└──────────────────────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────┐
              │  Axum Server     │
              │  - Session Store │
              │  - LLM API       │
              └──────────────────┘
```

## Components

### 1. Alpine.js Global Store

**Location:** `web/main.ts`

The Alpine.js store provides reactive global state accessible throughout the application via `$store.chat`.

**State:**
- `activeConversationId` - Currently active conversation
- `sessionId` - Server session ID
- `conversations` - Array of all conversations
- `status` - Connection status (idle | connecting | streaming | error)

**Methods:**
- `init()` - Load conversations from localStorage on startup
- `createConversation(sessionId)` - Create new conversation
- `switchConversation(id)` - Switch to different conversation
- `getStore()` - Get ConversationStore instance

**Usage in HTML:**
```html
<input type="hidden" name="session_id" x-bind:value="$store.chat?.sessionId || ''">
```

### 2. Conversation Store

**Location:** `web/stores/conversation-store.ts`

TypeScript class that manages conversation persistence and CRUD operations.

**Data Structures:**

```typescript
interface ConversationMessage {
  id: string;
  role: "user" | "assistant" | "tool" | "error";
  content: string;
  timestamp: number;
  toolCalls?: Array<{ id: string; name: string; arguments: string }>;
  toolResult?: { id: string; name: string; content: string; success: boolean };
}

interface Conversation {
  id: string;
  sessionId: string;
  title: string;
  messages: ConversationMessage[];
  createdAt: number;
  updatedAt: number;
}
```

**Methods:**
- `loadAll()` - Get all conversations sorted by update time
- `get(id)` - Get specific conversation
- `create(sessionId)` - Create new conversation
- `addMessage(conversationId, message)` - Add message to conversation
- `updateTitle(conversationId, title)` - Update conversation title
- `delete(conversationId)` - Delete conversation
- `clear()` - Clear all conversations (debugging)

**localStorage Key:** `chat_conversations`

### 3. chat-stream Web Component

**Location:** `web/components/chat-stream/chat-stream.ts`

The Web Component manages streaming state and renders messages.

**Key Changes:**
- Added `conversationStore` property
- Added `conversationId` property
- **Removed** `resetChatState()` call in `connect()` - state now persists
- Added `addUserMessage(content)` - Immediately displays user message
- Added `loadConversation(id)` - Loads conversation history
- Modified `handleDone()` - Saves assistant messages to ConversationStore

**Public Methods:**
- `addUserMessage(content: string)` - Add user message immediately
- `loadConversation(id: string)` - Load conversation from store
- `startStream(responseJson?: string)` - Start SSE streaming

## Message Flow

### Sending a Message

1. User types message and clicks send
2. **HTMX `hx-on--before-request`** fires:
   - Calls `chatStream.addUserMessage(msg)` - **User message appears immediately**
   - Sets `session_id` from Alpine store
3. Form submits to `/api/chat`
4. **HTMX `hx-on--after-request`** fires:
   - Updates Alpine store with `session_id` from response
   - Calls `chatStream.startStream(response)` to begin SSE
5. SSE events stream in and render in real-time
6. On `done` event, assistant message saved to ConversationStore

### Key Insight

The user message is added **before** the HTTP request, ensuring immediate feedback. The server session ID is synchronized after the response.

## Persistence Strategy

### localStorage Structure

```json
{
  "chat_conversations": [
    {
      "id": "conv_1234567890_abc123",
      "sessionId": "uuid-from-server",
      "title": "How do I implement state management...",
      "messages": [
        {
          "id": "msg-uuid-1",
          "role": "user",
          "content": "How do I implement state management?",
          "timestamp": 1234567890000
        },
        {
          "id": "msg-uuid-2",
          "role": "assistant",
          "content": "Here's how to implement state management...",
          "timestamp": 1234567891000,
          "toolCalls": [...]
        }
      ],
      "createdAt": 1234567890000,
      "updatedAt": 1234567891000
    }
  ],
  "last_active_conversation": "conv_1234567890_abc123"
}
```

### Persistence Points

1. **On conversation creation** - New conversation saved
2. **On user message** - Message added and saved
3. **On assistant message complete** - Message added and saved
4. **On conversation switch** - `last_active_conversation` updated

## Benefits

✅ **Immediate User Feedback** - User messages appear instantly before server response  
✅ **Persistent History** - Conversations survive page refresh via localStorage  
✅ **Multi-Conversation Support** - Users can maintain multiple chat threads  
✅ **Server Sync** - Server session IDs are tracked and synchronized  
✅ **Offline Resilience** - UI state persists even if connection fails  
✅ **HTMX Philosophy** - Server still controls data flow, client just maintains view state  
✅ **Progressive Enhancement** - Works without JavaScript (falls back to server-only mode)

## Testing

### Manual Testing Steps

1. **User Message Display**
   - Send a message
   - Verify user bubble appears immediately (before server response)
   - Verify message persists during streaming

2. **Persistence**
   - Send multiple messages
   - Refresh the page
   - Verify conversation history is restored
   - Check localStorage in DevTools

3. **Multi-Message Conversation**
   - Send 3-4 messages in sequence
   - Verify all messages (user and assistant) are displayed
   - Verify messages are in correct order

4. **localStorage Inspection**
   - Open DevTools → Application → localStorage
   - Find `chat_conversations` key
   - Verify JSON structure matches expected format

5. **Server Session Sync**
   - Send first message (creates session)
   - Send second message (reuses session)
   - Verify `session_id` is consistent in network tab

## Future Enhancements

- **Conversation Sidebar** - UI to switch between conversations
- **Conversation Search** - Search through message history
- **Export/Import** - Export conversations as JSON/Markdown
- **Conversation Limits** - Limit localStorage to last N conversations
- **Cloud Sync** - Optional backend sync for cross-device access
- **Conversation Sharing** - Share conversation via URL

## Troubleshooting

### User Message Not Showing

**Problem:** User message disappears after sending  
**Solution:** Verify `addUserMessage()` is called in `hx-on--before-request`

### Conversation Not Persisting

**Problem:** History lost on page refresh  
**Solution:** Check localStorage quota, verify `conversationStore.save()` is called

### Alpine Store Not Available

**Problem:** `$store.chat` is undefined  
**Solution:** Ensure Alpine.js is loaded and `alpine:init` event fired

### Session ID Not Syncing

**Problem:** New session created on every message  
**Solution:** Verify `session_id` hidden input is bound to `$store.chat.sessionId`

## Code References

- Alpine Store: [`web/main.ts:40-80`](web/main.ts)
- Conversation Store: [`web/stores/conversation-store.ts`](web/stores/conversation-store.ts)
- chat-stream Component: [`web/components/chat-stream/chat-stream.ts`](web/components/chat-stream/chat-stream.ts)
- HTMX Form: [`src/main.rs:221-260`](src/main.rs)
- Type Definitions: [`web/types/chat.ts:133-163`](web/types/chat.ts)
