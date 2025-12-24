# HTMX Streaming Chat Architecture Fix

## Problem Analysis

The current HTMX-based streaming chat application has several critical architectural issues:

### 1. **Runtime Exception**: Method Name Mismatch
- **Issue**: `chat-stream.ts` calls `scrollToBottom()` but method is named `smoothScrollToBottom()`
- **Impact**: Breaks streaming updates and triggers "connection error" spam
- **Location**: Lines 449, 509, 569 in `web/components/chat-stream/chat-stream.ts`

### 2. **Schema Mismatch**: Server vs Client Event Types
- **Server** (`src/normalized.rs`): 
  - `tool_call.delta` uses `call_index` + `arguments_delta`
  - `tool_result` uses `id` + `content`
- **Client** (`web/types/events.ts`):
  - Expects `index` + `arguments` 
  - Expects `tool_call_id` + `result`

### 3. **Transport Duplication**: Conflicting SSE Implementations
- **HTMX SSE Extension**: `static/vendor/htmx-sse.js` dispatches `htmx:sseMessage`
- **Custom SSE Wrapper**: `web/utils/sse.ts` with own EventSource lifecycle
- **Conflict**: Competing lifecycle/error semantics cause connection issues

### 4. **Flickering Updates**: Full Transcript Rebuilding
- **Current**: `renderTranscript()` rebuilds entire DOM on every update
- **Impact**: Visual flicker, poor performance, lost scroll position
- **Root Cause**: No keyed DOM management or incremental updates

### 5. **Tool Call Assembly Issues**
- **Problem**: Tool calls don't show immediately, arguments don't stream properly
- **Missing**: Placeholder blocks keyed by `call_index`, re-keying when `id` arrives

## Target Architecture

### Transport Layer: HTMX SSE Extension Only
```html
<!-- Internal SSE connector element -->
<div hx-ext="sse" 
     sse-connect="/api/chat/stream" 
     sse-close="done"
     style="display: none;">
</div>
```

**Benefits**:
- Single SSE transport managed by HTMX
- Automatic reconnection and error handling
- Standard `htmx:sseMessage` and `htmx:sseClose` events
- No custom EventSource wrapper needed

### Controller Layer: Stream State Management
```typescript
class StreamController {
  private turnAccumulator: ConversationTurn;
  private eventHandlers: Map<string, (data: any) => void>;
  
  handleNormalizedEvent(event: NormalizedEvent): void;
  applyToAccumulator(event: NormalizedEvent): void;
  notifyView(patches: DomPatch[]): void;
}
```

### View Layer: Keyed DOM Management
```typescript
class TranscriptView {
  private domIndex: Map<ItemKey, HTMLElement> = new Map();
  private patchQueue: DomPatch[] = [];
  private rafScheduled = false;
  
  appendItem(key: ItemKey, element: HTMLElement): void;
  patchItem(key: ItemKey, updates: Partial<ItemData>): void;
  flushPatches(): void; // RAF-scheduled
}
```

### Tool Call "Show Immediately" Flow

1. **First `tool_call.delta`**:
   ```typescript
   // Create placeholder immediately
   const key = `tool_idx_${call_index}`;
   const element = createToolCallBlock(key, { name: "Loading...", args: "" });
   transcriptView.appendItem(key, element);
   ```

2. **Stream `arguments_delta`**:
   ```typescript
   // Update arguments in-place
   transcriptView.patchItem(key, { 
     arguments: currentArgs + arguments_delta 
   });
   ```

3. **When `id` arrives**:
   ```typescript
   // Re-key without moving DOM
   const newKey = `tool_id_${id}`;
   transcriptView.rekeyItem(key, newKey);
   element.dataset.toolId = id;
   ```

4. **On `tool_result`**:
   ```typescript
   // Attach to existing block
   const key = `tool_id_${id}`;
   transcriptView.patchItem(key, { 
     result: content, 
     success: success 
   });
   ```

## Implementation Strategy

### Phase 1: Fix Critical Bugs
1. **Schema Alignment**: Update client types to match server
2. **Scroll Fix**: Replace `scrollToBottom()` calls
3. **Basic HTMX SSE**: Replace custom SSE with HTMX extension

### Phase 2: Keyed DOM System
1. **TranscriptView**: Implement keyed DOM management
2. **StreamController**: Centralize event handling
3. **Batched Updates**: RAF-scheduled DOM patches

### Phase 3: Tool Call Assembly
1. **Immediate Blocks**: Show tool calls on first delta
2. **Streaming Args**: Update arguments in-place
3. **Re-keying**: Smooth transition from index to id
4. **Result Attachment**: Connect results to existing blocks

### Phase 4: Polish & Testing
1. **Error Handling**: Improve SSE error recovery
2. **Scroll Behavior**: Smart auto-scroll detection
3. **Performance**: Optimize for high-velocity streams
4. **Database**: Ensure PGlite consistency

## Key Design Principles

### 1. **No Flicker**: Keyed DOM Updates
- Each chat item gets a stable key
- New items append once, updates patch in-place
- No full transcript rebuilding during streaming

### 2. **Correct Tool Streaming**: Immediate + Progressive
- Tool calls appear immediately on first delta
- Arguments stream into placeholder blocks
- Results attach to existing blocks by ID

### 3. **Deterministic Ordering**: Event Sequence Integrity
- Events apply in received order
- Stable keys prevent DOM reordering
- Consistent replay from database

### 4. **HTMX-Pure**: Single Transport
- HTMX SSE extension owns EventSource
- `chat-stream` consumes `htmx:sseMessage` only
- No competing transport implementations

## File Changes Required

### Core Architecture
- `web/components/chat-stream/chat-stream.ts` - Major refactor
- `web/types/events.ts` - Schema alignment
- `web/utils/sse.ts` - Remove chat streaming usage

### New Helper Classes
- `web/components/chat-stream/stream-controller.ts` - Event handling
- `web/components/chat-stream/transcript-view.ts` - DOM management
- `web/components/chat-stream/dom-patch-scheduler.ts` - Batched updates

### Testing & Verification
- `plans/streaming-verification-checklist.md` - Comprehensive test plan

## Success Metrics

1. **No Visual Flicker**: Smooth streaming without DOM rebuilds
2. **Immediate Tool Calls**: Blocks appear on first delta
3. **Correct Assembly**: Arguments stream, results attach properly
4. **Stable Replay**: Database conversations load identically
5. **Error Resilience**: No connection spam on transient errors
6. **Performance**: 60fps during high-velocity streaming

## Mermaid Architecture Diagram

```mermaid
graph TB
    ChatForm[Chat Form] -->|hx-post| API[POST /api/chat]
    API -->|stream_url| SSEConnector[SSE Connector Element]
    
    SSEConnector -->|hx-ext=sse| HTMXExt[HTMX SSE Extension]
    HTMXExt -->|htmx:sseMessage| StreamController[Stream Controller]
    
    StreamController -->|NormalizedEvent| TurnAccumulator[Turn Accumulator]
    StreamController -->|DomPatch[]| TranscriptView[Transcript View]
    
    TranscriptView -->|Map<ItemKey, HTMLElement>| DOMIndex[Keyed DOM Index]
    TranscriptView -->|RAF Batching| DomPatchBatch[DOM Patch Batch]
    
    DomPatchBatch -->|Update Elements| ChatTranscript[Chat Transcript DOM]
    
    subgraph "Tool Call Flow"
        ToolDelta[tool_call.delta] -->|call_index| PlaceholderBlock[tool_idx_N Block]
        PlaceholderBlock -->|arguments_delta| StreamingArgs[Streaming Arguments]
        StreamingArgs -->|id arrives| RekeyBlock[tool_id_X Block]
        RekeyBlock -->|tool_result| AttachResult[Attach Result]
    end
```

This architecture eliminates flicker, ensures correct tool streaming, maintains deterministic ordering, and uses HTMX SSE as the single transport mechanism.