# Implementation Roadmap

## Overview

This roadmap provides a step-by-step guide for implementing the comprehensive HTMX streaming chat architecture fix. The implementation is organized into phases to minimize risk and ensure each component can be tested independently.

## Phase 1: Foundation & Critical Fixes (Days 1-2)

### 1.1 Schema Alignment
**Priority**: Critical (blocks all streaming functionality)

**Files to Change**:
- `web/types/events.ts` - Update event interfaces
- `web/components/chat-stream/chat-stream.ts` - Update event handlers

**Implementation Steps**:
1. Update `ToolCallDeltaEvent` to use `call_index` and `arguments_delta`
2. Update `ToolResultEvent` to use `id` and `content`
3. Add missing `UsageEvent` interface
4. Update all event handlers to use new field names
5. Test with existing server to ensure compatibility

**Verification**:
- [ ] All SSE events parse correctly
- [ ] Tool calls receive proper field data
- [ ] No console errors about missing properties

### 1.2 Scroll Bug Fix
**Priority**: Critical (causes runtime exceptions)

**Files to Change**:
- `web/components/chat-stream/chat-stream.ts` - Fix method calls

**Implementation Steps**:
1. Replace all `this.scrollToBottom()` calls with `this.smoothScrollToBottom()`
2. Verify method exists and works correctly
3. Test scroll behavior during streaming

**Verification**:
- [ ] No runtime exceptions during streaming
- [ ] Smooth scrolling works as expected
- [ ] Auto-scroll only when user is near bottom

## Phase 2: HTMX SSE Transport (Days 3-4)

### 2.1 SSE Connector Element
**Priority**: High (eliminates transport duplication)

**Files to Change**:
- `web/components/chat-stream/chat-stream.ts` - Add SSE connector

**Implementation Steps**:
1. Create hidden SSE connector element in component template
2. Add `hx-ext="sse"` attribute
3. Dynamically set `sse-connect` attribute with stream URL
4. Add `sse-close="done"` for auto-disconnect

**Template Changes**:
```html
<div class="chat-stream">
  <!-- Hidden SSE connector -->
  <div class="sse-connector" 
       hx-ext="sse" 
       sse-close="done"
       style="display: none;">
  </div>
  <div class="chat-stream-transcript"></div>
  <div class="chat-stream-status"></div>
</div>
```

### 2.2 HTMX Event Listeners
**Priority**: High (replaces custom SSE wrapper)

**Implementation Steps**:
1. Add event listeners for `htmx:sseMessage`, `htmx:sseClose`, `htmx:sseError`
2. Parse SSE data into `NormalizedEvent` objects
3. Remove `SSEConnection` usage from chat streaming
4. Keep `web/utils/sse.ts` for other potential uses

**Verification**:
- [ ] Single EventSource connection active
- [ ] All normalized events received correctly
- [ ] Automatic reconnection works via HTMX
- [ ] No duplicate event subscriptions

## Phase 3: Keyed DOM System (Days 5-7)

### 3.1 DOM Patch Scheduler
**Priority**: High (foundation for flicker-free updates)

**Files to Create**:
- `web/components/chat-stream/dom-patch-scheduler.ts`

**Implementation Steps**:
1. Create `DomPatchScheduler` class with RAF-based batching
2. Implement frame budget management (12ms for 60fps)
3. Add patch queue with priority handling
4. Support for append, patch, rekey, remove operations

**Interface**:
```typescript
export interface DomPatch {
  type: 'append' | 'patch' | 'rekey' | 'remove';
  key: string;
  element?: HTMLElement;
  updates?: Record<string, any>;
  newKey?: string;
}

export class DomPatchScheduler {
  schedule(patches: DomPatch[]): void;
  flush(): void;
  cancel(): void;
}
```

### 3.2 Transcript View
**Priority**: High (eliminates flicker)

**Files to Create**:
- `web/components/chat-stream/transcript-view.ts`

**Implementation Steps**:
1. Create `TranscriptView` class with keyed DOM management
2. Implement `Map<ItemKey, HTMLElement>` for O(1) lookups
3. Add append-once, patch-only update methods
4. Integrate with `DomPatchScheduler` for batched updates
5. Add smart auto-scroll with near-bottom detection

**Key Methods**:
```typescript
export class TranscriptView {
  appendItem(key: string, element: HTMLElement): void;
  patchItem(key: string, updates: Record<string, any>): void;
  rekeyItem(oldKey: string, newKey: string): void;
  removeItem(key: string): void;
  shouldAutoScroll(): boolean;
  smoothScrollToBottom(): void;
}
```

### 3.3 Item Key Generation
**Priority**: High (ensures stable DOM keys)

**Implementation Steps**:
1. Create deterministic key generation for all chat item types
2. Ensure keys are stable across page reloads
3. Support for placeholder keys that can be re-keyed

**Key Generation Logic**:
```typescript
export function generateItemKey(item: ChatItem, index: number): string {
  switch (item.kind) {
    case 'message':
      return `msg_${item.role}_${index}`;
    case 'tool_call':
      return item.id ? `tool_id_${item.id}` : `tool_idx_${item.callIndex}`;
    case 'tool_result':
      return `result_${item.toolCallId}`;
    default:
      return `item_${index}`;
  }
}
```

## Phase 4: Stream Controller (Days 8-9)

### 4.1 Stream Controller Class
**Priority**: High (centralizes event handling)

**Files to Create**:
- `web/components/chat-stream/stream-controller.ts`

**Implementation Steps**:
1. Create `StreamController` class to manage all event handling
2. Implement turn accumulator for database persistence
3. Add event-specific handlers for each `NormalizedEvent` type
4. Generate DOM patches for view updates
5. Coordinate with `TranscriptView` for DOM updates

**Architecture**:
```typescript
export class StreamController {
  constructor(config: {
    onViewUpdate: (patches: DomPatch[]) => void;
    onError: (error: StreamError) => void;
    onComplete: (turn: ConversationTurn) => void;
  });
  
  handleNormalizedEvent(event: NormalizedEvent): void;
  reset(): void;
  getCurrentTurn(): ConversationTurn;
}
```

### 4.2 Event Handler Migration
**Priority**: High (removes scattered event logic)

**Implementation Steps**:
1. Move all event handling from `chat-stream.ts` to `StreamController`
2. Update event handlers to generate DOM patches instead of direct DOM manipulation
3. Maintain state in turn accumulator for database persistence
4. Add proper error handling and recovery

## Phase 5: Tool Call Assembly (Days 10-11)

### 5.1 Tool Call Assembler
**Priority**: High (fixes tool call streaming)

**Files to Create**:
- `web/components/chat-stream/tool-call-assembler.ts`

**Implementation Steps**:
1. Create `ToolCallAssembler` class for complex tool call lifecycle
2. Implement immediate placeholder creation on first delta
3. Add progressive argument streaming into existing blocks
4. Support re-keying from `tool_idx_N` to `tool_id_X`
5. Handle tool result attachment to existing blocks

**Key Features**:
```typescript
export class ToolCallAssembler {
  handleDelta(data: ToolCallDeltaData): DomPatch[];
  handleComplete(data: ToolCallCompleteData): DomPatch[];
  handleResult(data: ToolResultData): DomPatch[];
  
  private rekeyCall(callIndex: number, id: string): DomPatch[];
  private createToolCallElement(key: string, state: ToolCallState): HTMLElement;
}
```

### 5.2 Tool Call Flow Implementation
**Priority**: High (ensures correct tool streaming)

**Implementation Steps**:
1. Create placeholder blocks immediately on first `tool_call.delta`
2. Stream `arguments_delta` into existing blocks progressively
3. Re-key blocks when `id` arrives (without DOM movement)
4. Attach results to existing blocks by ID
5. Handle edge cases (missing results, multiple calls, etc.)

## Phase 6: Integration & Refactoring (Days 12-13)

### 6.1 Chat Stream Component Refactor
**Priority**: High (integrates all components)

**Files to Change**:
- `web/components/chat-stream/chat-stream.ts` - Major refactor

**Implementation Steps**:
1. Replace existing event handling with `StreamController`
2. Replace `renderTranscript()` with `TranscriptView` operations
3. Integrate HTMX SSE event listeners
4. Remove custom SSE connection logic
5. Add component lifecycle management

**New Architecture**:
```typescript
export class ChatStream extends HTMLElement {
  private streamController: StreamController;
  private transcriptView: TranscriptView;
  private toolCallAssembler: ToolCallAssembler;
  private sseConnector: HTMLElement;
  
  private setupComponents(): void;
  private setupSSEListeners(): void;
  private handleSSEMessage(event: CustomEvent): void;
}
```

### 6.2 Error Handling Improvements
**Priority**: Medium (improves user experience)

**Implementation Steps**:
1. Distinguish between recoverable and fatal errors
2. Stop showing UI errors for transient connection issues
3. Implement proper error recovery via HTMX SSE
4. Add error classification and handling logic

## Phase 7: Database Consistency (Days 14-15)

### 7.1 PGlite Integration Updates
**Priority**: Medium (ensures data consistency)

**Files to Change**:
- `web/components/chat-stream/chat-stream.ts` - Update database operations

**Implementation Steps**:
1. Ensure persisted turns match new keyed structure
2. Update conversation loading to use same keys as live streaming
3. Verify tool call/result relationships are maintained
4. Test conversation replay matches live streaming exactly

### 7.2 Migration Support
**Priority**: Low (backward compatibility)

**Implementation Steps**:
1. Handle existing conversations with old structure
2. Migrate data format if necessary
3. Ensure no data loss during transition

## Phase 8: Testing & Optimization (Days 16-18)

### 8.1 Comprehensive Testing
**Priority**: High (ensures quality)

**Testing Areas**:
- [ ] No flicker during message streaming
- [ ] Correct tool call assembly and result attachment
- [ ] Stable conversation replay from database
- [ ] No reconnection spam on temporary errors
- [ ] Proper scroll behavior and performance
- [ ] Cross-browser compatibility
- [ ] Memory and CPU performance
- [ ] Error handling and recovery

### 8.2 Performance Optimization
**Priority**: Medium (ensures smooth experience)

**Optimization Areas**:
- [ ] Frame rate monitoring and optimization
- [ ] Memory usage optimization
- [ ] Batch size tuning for different stream velocities
- [ ] Incremental markdown parsing improvements

## Risk Mitigation

### High-Risk Areas
1. **HTMX SSE Integration**: Test thoroughly across browsers
2. **DOM Patch Scheduler**: Ensure frame budget is respected
3. **Tool Call Re-keying**: Verify no DOM nodes are lost
4. **Database Migration**: Backup and test data consistency

### Rollback Plan
1. Keep existing `chat-stream.ts` as backup
2. Feature flags for new vs old implementation
3. Gradual rollout with monitoring
4. Quick revert capability if issues arise

### Testing Strategy
1. **Unit Tests**: Each component class individually
2. **Integration Tests**: Component interaction and event flow
3. **Performance Tests**: Frame rate and memory usage
4. **Manual Testing**: Real-world usage scenarios

## Success Metrics

### Technical Metrics
- **Zero Flicker**: No visible DOM rebuilds during streaming
- **60fps Performance**: Smooth streaming at high velocity
- **Memory Stability**: No memory leaks over extended use
- **Error Resilience**: Graceful handling of connection issues

### User Experience Metrics
- **Immediate Responsiveness**: Tool calls appear instantly
- **Smooth Scrolling**: Natural scroll behavior
- **Reliable Persistence**: Conversations load identically
- **Error Recovery**: Seamless reconnection after network issues

This roadmap provides a structured approach to implementing the comprehensive architecture fix while minimizing risk and ensuring each component can be thoroughly tested before integration.