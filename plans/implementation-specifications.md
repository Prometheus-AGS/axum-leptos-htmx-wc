# Implementation Specifications

## 1. Schema Alignment (`web/types/events.ts`)

### Current Issues
```typescript
// Server (src/normalized.rs) sends:
ToolCallDelta {
  call_index: usize,
  arguments_delta: Option<String>
}
ToolResult {
  id: String,
  content: String
}

// Client (web/types/events.ts) expects:
ToolCallDeltaEvent {
  data: {
    index: number;
    arguments?: string;
  }
}
ToolResultEvent {
  data: {
    tool_call_id: string;
    result: string;
  }
}
```

### Fixed Schema
```typescript
export interface ToolCallDeltaEvent {
  type: "tool_call.delta";
  data: {
    call_index: number;        // Match server field name
    id?: string;
    name?: string;
    arguments_delta?: string;  // Match server field name
  };
}

export interface ToolResultEvent {
  type: "tool_result";
  data: {
    id: string;               // Match server field name
    name: string;
    content: string;          // Match server field name
    success: boolean;
  };
}

// Add missing Usage event
export interface UsageEvent {
  type: "usage";
  data: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}
```

## 2. Stream Controller (`web/components/chat-stream/stream-controller.ts`)

### Purpose
Centralized event handling and state management, replacing scattered event logic in `chat-stream.ts`.

### Interface
```typescript
export interface StreamControllerConfig {
  onViewUpdate: (patches: DomPatch[]) => void;
  onError: (error: StreamError) => void;
  onComplete: (turn: ConversationTurn) => void;
}

export class StreamController {
  private turnAccumulator: ConversationTurn;
  private toolCallsInProgress: Map<number, ToolCallAccumulator>;
  private config: StreamControllerConfig;

  constructor(config: StreamControllerConfig);
  
  // Main event handler
  handleNormalizedEvent(event: NormalizedEvent): void;
  
  // Event-specific handlers
  private handleMessageDelta(data: { text: string }): void;
  private handleThinkingDelta(data: { text: string }): void;
  private handleReasoningDelta(data: { text: string }): void;
  private handleToolCallDelta(data: ToolCallDeltaData): void;
  private handleToolCallComplete(data: ToolCallCompleteData): void;
  private handleToolResult(data: ToolResultData): void;
  private handleUsage(data: UsageData): void;
  private handleDone(): void;
  
  // State management
  reset(): void;
  getCurrentTurn(): ConversationTurn;
}
```

### Key Features
- **Immediate Tool Call Creation**: Creates DOM patches for tool calls on first delta
- **Progressive Assembly**: Streams arguments into existing blocks
- **Re-keying Logic**: Handles transition from `call_index` to `id`
- **State Accumulation**: Builds complete turn for database persistence

## 3. Transcript View (`web/components/chat-stream/transcript-view.ts`)

### Purpose
Keyed DOM management system that eliminates flicker through append-once + patch-only updates.

### Interface
```typescript
export type ItemKey = string;

export interface DomPatch {
  type: 'append' | 'patch' | 'rekey' | 'remove';
  key: ItemKey;
  element?: HTMLElement;
  updates?: Record<string, any>;
  newKey?: ItemKey;
}

export class TranscriptView {
  private container: HTMLElement;
  private domIndex: Map<ItemKey, HTMLElement> = new Map();
  private patchQueue: DomPatch[] = [];
  private rafScheduled = false;
  private scrollBehavior: ScrollBehavior;

  constructor(container: HTMLElement);
  
  // Core operations
  queuePatch(patch: DomPatch): void;
  flushPatches(): void;
  
  // Convenience methods
  appendItem(key: ItemKey, element: HTMLElement): void;
  patchItem(key: ItemKey, updates: Record<string, any>): void;
  rekeyItem(oldKey: ItemKey, newKey: ItemKey): void;
  removeItem(key: ItemKey): void;
  
  // Scroll management
  shouldAutoScroll(): boolean;
  smoothScrollToBottom(): void;
  
  // Cleanup
  clear(): void;
}
```

### Key Features
- **Keyed DOM Index**: `Map<ItemKey, HTMLElement>` for O(1) element lookup
- **Batched Updates**: RAF-scheduled patch application for smooth 60fps
- **Smart Scrolling**: Auto-scroll only when user is near bottom
- **Re-keying Support**: Change element keys without DOM manipulation

## 4. DOM Patch Scheduler (`web/components/chat-stream/dom-patch-scheduler.ts`)

### Purpose
High-performance batched DOM updates with frame budget management.

### Interface
```typescript
export interface SchedulerConfig {
  maxFrameBudgetMs: number;  // Default: 12ms for 60fps
  minFrameDelay: number;     // Default: 16ms
  batchSize: number;         // Default: 10 patches per frame
}

export class DomPatchScheduler {
  private config: SchedulerConfig;
  private patchQueue: DomPatch[] = [];
  private rafId: number | null = null;
  private isProcessing = false;

  constructor(config: Partial<SchedulerConfig> = {});
  
  schedule(patches: DomPatch[]): void;
  flush(): void;
  cancel(): void;
  
  private processBatch(startTime: number): void;
  private applyPatch(patch: DomPatch): void;
}
```

### Key Features
- **Frame Budget**: Respects 12ms budget for smooth 60fps
- **Batch Processing**: Processes multiple patches per frame
- **Overflow Handling**: Continues in next frame if budget exceeded
- **Priority Queue**: Critical updates (scroll, errors) processed first

## 5. Tool Call Assembler (`web/components/chat-stream/tool-call-assembler.ts`)

### Purpose
Handles the complex tool call lifecycle with immediate display and progressive assembly.

### Interface
```typescript
export interface ToolCallState {
  callIndex: number;
  id?: string;
  name?: string;
  arguments: string;
  status: 'streaming' | 'complete' | 'error';
  result?: ToolResult;
}

export class ToolCallAssembler {
  private activeCalls: Map<number, ToolCallState> = new Map();
  private callsByIndex: Map<number, string> = new Map(); // call_index -> key
  private callsById: Map<string, string> = new Map();    // id -> key

  // Main handlers
  handleDelta(data: ToolCallDeltaData): DomPatch[];
  handleComplete(data: ToolCallCompleteData): DomPatch[];
  handleResult(data: ToolResultData): DomPatch[];
  
  // Key management
  private generatePlaceholderKey(callIndex: number): string;
  private generateFinalKey(id: string): string;
  private rekeyCall(callIndex: number, id: string): DomPatch[];
  
  // DOM element creation
  private createToolCallElement(key: string, state: ToolCallState): HTMLElement;
  private updateToolCallElement(element: HTMLElement, updates: Partial<ToolCallState>): void;
}
```

### Key Features
- **Immediate Display**: Creates placeholder on first delta
- **Progressive Assembly**: Streams arguments into existing block
- **Smart Re-keying**: Transitions from `tool_idx_N` to `tool_id_X` seamlessly
- **Result Attachment**: Connects results to existing blocks by ID

## 6. HTMX SSE Integration

### Current Architecture Problem
```typescript
// Current: Competing SSE implementations
this.connection = new SSEConnection({
  url: this.streamUrl,
  handlers: { onNormalizedEvent: (event) => this.handleEvent(event) }
});
```

### New HTMX-Pure Architecture
```html
<!-- Internal SSE connector (hidden) -->
<div class="sse-connector" 
     hx-ext="sse" 
     sse-connect="/api/chat/stream" 
     sse-close="done"
     style="display: none;">
</div>
```

```typescript
// Event listener approach
private setupSSEListeners(): void {
  // Listen for HTMX SSE events
  this.addEventListener('htmx:sseMessage', this.handleSSEMessage);
  this.addEventListener('htmx:sseClose', this.handleSSEClose);
  this.addEventListener('htmx:sseError', this.handleSSEError);
}

private handleSSEMessage = (event: CustomEvent): void => {
  const { data, type } = event.detail;
  const normalizedEvent = this.parseSSEData(data, type);
  if (normalizedEvent) {
    this.streamController.handleNormalizedEvent(normalizedEvent);
  }
};
```

### Benefits
- **Single Transport**: HTMX SSE extension manages EventSource
- **Automatic Reconnection**: Built-in retry logic with exponential backoff
- **Error Handling**: Proper `htmx:sseError` events without spam
- **Lifecycle Management**: Clean `htmx:sseClose` handling

## 7. Refactored Chat Stream Component

### New Structure
```typescript
export class ChatStream extends HTMLElement {
  // Core components
  private streamController: StreamController;
  private transcriptView: TranscriptView;
  private toolCallAssembler: ToolCallAssembler;
  private sseConnector: HTMLElement;
  
  // State (minimal)
  private conversationId: string | null = null;
  private isStreaming = false;
  
  // Lifecycle
  connectedCallback(): void {
    this.render();
    this.setupComponents();
    this.setupSSEListeners();
  }
  
  // Component setup
  private setupComponents(): void {
    this.streamController = new StreamController({
      onViewUpdate: (patches) => this.transcriptView.queuePatch(...patches),
      onError: (error) => this.handleStreamError(error),
      onComplete: (turn) => this.saveTurn(turn)
    });
    
    this.transcriptView = new TranscriptView(
      this.querySelector('.chat-stream-transcript')!
    );
    
    this.toolCallAssembler = new ToolCallAssembler();
  }
  
  // SSE management
  startStream(responseJson?: string): void {
    const streamUrl = this.extractStreamUrl(responseJson);
    this.sseConnector.setAttribute('sse-connect', streamUrl);
    this.isStreaming = true;
  }
  
  private handleSSEMessage = (event: CustomEvent): void => {
    // Delegate to stream controller
    const normalizedEvent = this.parseSSEEvent(event);
    if (normalizedEvent) {
      this.streamController.handleNormalizedEvent(normalizedEvent);
    }
  };
}
```

### Key Changes
- **Separation of Concerns**: Controller, View, and Assembler are distinct
- **HTMX SSE Only**: No custom EventSource wrapper
- **Keyed DOM**: No more `renderTranscript()` rebuilding
- **Event-Driven**: Components communicate via patches and events

## 8. Error Handling Improvements

### Current Problems
- Connection errors trigger error bubbles in transcript
- Transient network issues cause reconnection spam
- No distinction between recoverable and fatal errors

### New Error Strategy
```typescript
export interface StreamError {
  type: 'connection' | 'parsing' | 'server' | 'fatal';
  message: string;
  recoverable: boolean;
  retryAfter?: number;
}

export class ErrorHandler {
  private errorCount = 0;
  private lastErrorTime = 0;
  
  handleSSEError(event: CustomEvent): StreamError {
    const { error, source } = event.detail;
    
    // Classify error
    if (this.isTransientNetworkError(error)) {
      return {
        type: 'connection',
        message: 'Connection interrupted',
        recoverable: true,
        retryAfter: this.calculateBackoff()
      };
    }
    
    // Don't show UI errors for recoverable connection issues
    if (error.type === 'connection' && error.recoverable) {
      console.debug('[chat-stream] Transient connection error, retrying...');
      return error;
    }
    
    // Show UI errors only for fatal issues
    return {
      type: 'fatal',
      message: 'Unable to connect to chat service',
      recoverable: false
    };
  }
}
```

## 9. Database Consistency

### Key Requirements
- Persisted turns must match new keyed structure
- Conversation replay must be identical to live streaming
- Tool calls and results maintain proper relationships

### Implementation
```typescript
// Ensure keys are deterministic and stable
export function generateItemKey(item: ChatItem, index: number): ItemKey {
  switch (item.kind) {
    case 'message':
      return `msg_${item.role}_${index}`;
    case 'thinking':
      return `thinking_${index}`;
    case 'reasoning':
      return `reasoning_${index}`;
    case 'tool_call':
      return item.id ? `tool_id_${item.id}` : `tool_idx_${item.callIndex}`;
    case 'tool_result':
      return `result_${item.toolCallId}`;
    default:
      return `item_${index}`;
  }
}

// Replay conversations with same keys as live streaming
async loadConversation(id: string): Promise<void> {
  const history = await pgliteStore.loadConversation(id);
  
  // Use same key generation as live streaming
  for (const [index, item] of history.items.entries()) {
    const key = generateItemKey(item, index);
    const element = this.createItemElement(item);
    this.transcriptView.appendItem(key, element);
  }
}
```

## 10. Performance Optimizations

### Streaming Velocity Adaptation
```typescript
// Adapt batch sizes based on stream velocity
private adaptBatchSize(velocity: number): number {
  if (velocity > 1000) return 100;  // High velocity: larger batches
  if (velocity > 500) return 50;    // Medium velocity: balanced
  return 20;                        // Low velocity: smaller batches for typing effect
}
```

### Memory Management
```typescript
// Clean up completed streams
private cleanup(): void {
  this.streamController.reset();
  this.transcriptView.clear();
  this.toolCallAssembler.reset();
  
  // Clear RAF callbacks
  this.domPatchScheduler.cancel();
}
```

### Incremental Markdown Parsing
```typescript
// Only re-parse unstable markdown boundaries
private parseMarkdownIncremental(fullText: string): string {
  const stableBoundary = this.findStableMarkdownBoundary(fullText);
  const unstableText = fullText.slice(stableBoundary);
  const unstableHtml = renderMarkdown(unstableText);
  
  return this.cachedStableHtml + unstableHtml;
}
```

This comprehensive refactoring addresses all identified issues while maintaining backward compatibility and ensuring smooth, flicker-free streaming performance.