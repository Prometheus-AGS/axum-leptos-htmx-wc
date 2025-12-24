# Streaming Verification Checklist

## Pre-Implementation Verification

### 1. Schema Alignment ✓
- [ ] Server `src/normalized.rs` uses `call_index` and `arguments_delta`
- [ ] Client `web/types/events.ts` matches server field names exactly
- [ ] `ToolResult` uses `id` and `content` (not `tool_call_id` and `result`)
- [ ] `Usage` event type exists in client types
- [ ] All event type guards updated for new schema

### 2. Current Issues Documented ✓
- [ ] `scrollToBottom()` vs `smoothScrollToBottom()` method name mismatch identified
- [ ] Transport duplication between HTMX SSE and custom SSE confirmed
- [ ] Full transcript rebuilding causing flicker documented
- [ ] Tool call assembly issues mapped to specific event flow

## Implementation Verification

### 3. HTMX SSE Transport
- [ ] Internal SSE connector element created with `hx-ext="sse"`
- [ ] `sse-connect` attribute dynamically set to stream URL
- [ ] `sse-close="done"` attribute configured for auto-disconnect
- [ ] `htmx:sseMessage` event listener handles all normalized events
- [ ] `htmx:sseClose` event listener cleans up connection state
- [ ] `htmx:sseError` event listener handles errors without UI spam
- [ ] Custom `SSEConnection` class no longer used for chat streaming
- [ ] `web/utils/sse.ts` dependency removed from chat-stream component

### 4. Stream Controller
- [ ] `StreamController` class centralizes all event handling
- [ ] `handleNormalizedEvent()` method routes events to specific handlers
- [ ] Turn accumulator builds complete conversation turn
- [ ] Tool call state managed with `Map<number, ToolCallState>`
- [ ] DOM patches generated for all state changes
- [ ] Error handling distinguishes recoverable vs fatal errors
- [ ] Memory cleanup on stream completion

### 5. Transcript View (Keyed DOM)
- [ ] `TranscriptView` class manages DOM with `Map<ItemKey, HTMLElement>`
- [ ] `appendItem()` adds new elements once (no rebuilding)
- [ ] `patchItem()` updates existing elements in-place
- [ ] `rekeyItem()` changes keys without DOM manipulation
- [ ] Batched DOM updates via RAF scheduler
- [ ] Frame budget respected (12ms for 60fps)
- [ ] Smart auto-scroll only when user near bottom
- [ ] Scroll position preserved during updates

### 6. Tool Call Assembly
- [ ] Immediate placeholder creation on first `tool_call.delta`
- [ ] Placeholder keyed as `tool_idx_{call_index}`
- [ ] Arguments stream into existing block progressively
- [ ] Re-keying to `tool_id_{id}` when ID arrives (no DOM move)
- [ ] Tool results attach to existing blocks by ID
- [ ] No orphaned tool results or duplicate blocks
- [ ] Tool call status updates (streaming → complete → result)

### 7. DOM Patch Scheduler
- [ ] `DomPatchScheduler` batches updates for smooth performance
- [ ] RAF-based scheduling maintains 60fps target
- [ ] Frame budget monitoring prevents jank
- [ ] Patch queue overflow handled gracefully
- [ ] Priority patches (errors, scroll) processed first
- [ ] Cleanup cancels pending RAF callbacks

## Functional Verification

### 8. No Flicker During Streaming
- [ ] **Test**: Start new conversation, send message
- [ ] **Verify**: Assistant response streams smoothly without DOM rebuilds
- [ ] **Check**: No visible flicker or content jumps
- [ ] **Measure**: Consistent 60fps during high-velocity streaming
- [ ] **Edge Case**: Very long messages (>10k chars) stream smoothly
- [ ] **Edge Case**: Rapid message bursts don't cause frame drops

### 9. Correct Tool Call Flow
- [ ] **Test**: Send message that triggers tool calls
- [ ] **Verify**: Tool call block appears immediately on first delta
- [ ] **Check**: Arguments stream into placeholder block progressively
- [ ] **Verify**: Block re-keys from `tool_idx_N` to `tool_id_X` seamlessly
- [ ] **Check**: Tool result attaches to existing block (no new block)
- [ ] **Edge Case**: Multiple simultaneous tool calls handled correctly
- [ ] **Edge Case**: Tool call without result doesn't break UI

### 10. Stable Conversation Replay
- [ ] **Test**: Load existing conversation from database
- [ ] **Verify**: Same DOM structure as live streaming
- [ ] **Check**: Tool calls and results properly associated
- [ ] **Verify**: Scroll position and layout identical
- [ ] **Edge Case**: Conversations with incomplete tool calls load correctly
- [ ] **Edge Case**: Very long conversations (>100 messages) load efficiently

### 11. Error Handling
- [ ] **Test**: Disconnect network during streaming
- [ ] **Verify**: No error bubbles appear in transcript
- [ ] **Check**: HTMX SSE handles reconnection automatically
- [ ] **Verify**: Streaming resumes after reconnection
- [ ] **Test**: Server returns malformed SSE data
- [ ] **Verify**: Parsing errors logged but don't break UI
- [ ] **Edge Case**: Rapid connect/disconnect cycles handled gracefully

### 12. Scroll Behavior
- [ ] **Test**: Stream long message while scrolled to bottom
- [ ] **Verify**: Auto-scroll keeps bottom visible
- [ ] **Test**: Stream message while scrolled up in history
- [ ] **Verify**: No auto-scroll, position preserved
- [ ] **Check**: Smooth scroll animation (not instant jump)
- [ ] **Edge Case**: Scroll during tool call assembly works correctly

## Performance Verification

### 13. Memory Management
- [ ] **Test**: Stream 50+ messages in single conversation
- [ ] **Verify**: Memory usage remains stable
- [ ] **Check**: DOM nodes cleaned up properly
- [ ] **Verify**: Event listeners removed on disconnect
- [ ] **Tool**: Use browser DevTools to monitor memory

### 14. CPU Performance
- [ ] **Test**: High-velocity streaming (>1000 chars/sec)
- [ ] **Verify**: CPU usage remains reasonable (<50%)
- [ ] **Check**: No blocking operations on main thread
- [ ] **Measure**: Frame rate stays above 55fps
- [ ] **Tool**: Use Performance tab in DevTools

### 15. Network Efficiency
- [ ] **Test**: Monitor SSE connection during streaming
- [ ] **Verify**: Single EventSource connection active
- [ ] **Check**: No duplicate event subscriptions
- [ ] **Verify**: Connection closes cleanly on completion
- [ ] **Tool**: Use Network tab to verify SSE behavior

## Database Consistency

### 16. PGlite Integration
- [ ] **Test**: Complete conversation turn saves correctly
- [ ] **Verify**: All chunks (messages, thinking, tools) persisted
- [ ] **Check**: Tool call/result relationships maintained
- [ ] **Verify**: Conversation replay matches live streaming exactly
- [ ] **Edge Case**: Interrupted streams save partial state correctly

### 17. Data Integrity
- [ ] **Test**: Load conversation after browser refresh
- [ ] **Verify**: All content restored accurately
- [ ] **Check**: Tool calls show correct arguments and results
- [ ] **Verify**: Message order and timestamps preserved
- [ ] **Edge Case**: Corrupted database entries handled gracefully

## Browser Compatibility

### 18. Cross-Browser Testing
- [ ] **Chrome**: All features work correctly
- [ ] **Firefox**: SSE and DOM updates function properly
- [ ] **Safari**: HTMX SSE extension compatible
- [ ] **Edge**: Performance meets standards
- [ ] **Mobile**: Touch scrolling and responsive layout work

### 19. Accessibility
- [ ] **Screen Reader**: `aria-live="polite"` announces new messages
- [ ] **Keyboard**: Tab navigation works through tool calls
- [ ] **Focus**: Focus management during streaming updates
- [ ] **Contrast**: All text meets WCAG standards

## Regression Testing

### 20. Existing Features
- [ ] **Conversation Sidebar**: Still updates correctly
- [ ] **Theme Switching**: Works during streaming
- [ ] **Copy Buttons**: Function on all message types
- [ ] **Markdown Rendering**: Code blocks, lists, etc. render correctly
- [ ] **Token Counter**: Updates accurately during streaming

### 21. Edge Cases
- [ ] **Empty Messages**: Handled gracefully
- [ ] **Unicode Content**: Emoji and special characters work
- [ ] **Large Payloads**: >1MB responses don't break streaming
- [ ] **Malformed JSON**: Parsing errors don't crash component
- [ ] **Rapid User Actions**: Multiple quick messages handled correctly

## Final Acceptance Criteria

### ✅ Success Metrics
1. **Zero Flicker**: No visible DOM rebuilds during streaming
2. **Immediate Tool Calls**: Blocks appear on first delta
3. **Correct Assembly**: Arguments stream, results attach properly
4. **Stable Replay**: Database conversations identical to live
5. **Error Resilience**: No UI spam on connection issues
6. **60fps Performance**: Smooth streaming at high velocity

### ❌ Failure Conditions
1. Any visible flicker during message streaming
2. Tool calls that don't appear immediately
3. Orphaned tool results or duplicate blocks
4. Database replay different from live streaming
5. Error bubbles in transcript for connection issues
6. Frame rate drops below 55fps during streaming

## Testing Protocol

### Manual Testing Steps
1. **Setup**: Fresh browser, clear storage
2. **Basic Flow**: Send message, verify smooth streaming
3. **Tool Flow**: Trigger tool calls, verify assembly
4. **Error Flow**: Disconnect network, verify recovery
5. **Performance**: Monitor DevTools during high-velocity stream
6. **Database**: Refresh page, verify conversation reload

### Automated Testing
- Unit tests for each component class
- Integration tests for event flow
- Performance benchmarks for streaming velocity
- Memory leak detection tests

This checklist ensures the refactored architecture delivers on all requirements while maintaining backward compatibility and performance standards.