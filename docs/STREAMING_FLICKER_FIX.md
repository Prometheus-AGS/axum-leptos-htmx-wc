# Streaming Flicker Fix

## Problem

The streaming UI was experiencing severe flickering and glitching during message streaming, making it difficult to read the content as it arrived. The screen would flash and jump rapidly, creating a poor user experience.

## Root Cause

The debouncing logic was calling `renderTranscript()` on every batch flush, which:
1. **Re-rendered the entire transcript** (all messages, tool calls, thinking blocks, etc.)
2. **Re-checked the thinking badge condition** on every update
3. **Re-parsed all HTML** for every message bubble
4. **Caused layout thrashing** as the browser recalculated positions for all elements

This resulted in 30-60 full DOM re-renders per second during active streaming.

## Solution

### 1. Direct DOM Updates (No Full Re-renders)

Instead of calling `renderTranscript()` on every batch, we now:
- **Update only the streaming element** directly via `innerHTML`
- **Keep the rest of the transcript unchanged**
- **Only do full re-renders** when a new item is added to the transcript

```typescript
// BEFORE (flickering):
private handleMessageDelta(text: string): void {
  this.streamingOptimizer.bufferTextChunk(streamId, text, (flushedText) => {
    this.state.streamingText += flushedText;
    // ... update state ...
    this.renderTranscript(); // ❌ Re-renders EVERYTHING
  });
}

// AFTER (smooth):
private handleMessageDelta(text: string): void {
  this.streamingOptimizer.bufferTextChunk(streamId, text, (flushedText) => {
    this.state.streamingText += flushedText;
    // ... update state ...
    
    if (isNewItem) {
      this.renderTranscript(); // Only on first chunk
    } else {
      this.updateStreamingMessageDOM(html); // ✅ Update only streaming element
    }
  });
}
```

### 2. Targeted DOM Updates

New helper methods that update only the streaming content:

```typescript
/**
 * Update only the streaming message DOM (avoids full re-render flicker).
 */
private updateStreamingMessageDOM(html: string): void {
  const messageBubbles = this.transcriptEl.querySelectorAll('.chat-message.assistant');
  const lastBubble = messageBubbles[messageBubbles.length - 1];
  
  if (lastBubble) {
    const proseDiv = lastBubble.querySelector('.prose');
    if (proseDiv) {
      proseDiv.innerHTML = html; // Direct update
      this.scrollToBottom();
    }
  }
}
```

Similar methods for:
- `updateStreamingThinkingDOM()` - Updates thinking blocks
- `updateStreamingReasoningDOM()` - Updates reasoning blocks

### 3. Increased Batch Sizes

Reduced update frequency by increasing batch sizes:

```typescript
// BEFORE:
textBatchSize: 20,        // 20 chars per update
textBatchDelay: 16,       // 60fps
markdownBatchSize: 40,
thinkingBatchSize: 60,
minFrameDelay: 16,

// AFTER:
textBatchSize: 50,        // 50 chars per update (2.5x less frequent)
textBatchDelay: 32,       // 30fps (smoother than 60fps for text)
markdownBatchSize: 80,    // 2x increase
thinkingBatchSize: 100,   // 1.67x increase
minFrameDelay: 32,        // 30fps target
```

**Why 30fps is better than 60fps for text streaming:**
- Text streaming doesn't need 60fps (not a game or animation)
- 30fps is perfectly smooth for reading
- Less frequent updates = less CPU usage
- Larger batches = more coherent text chunks

### 4. Thinking Badge Optimization

The thinking badge is now only checked during `renderTranscript()` calls, not on every batch flush:

```typescript
private renderTranscript(): void {
  const html = this.state.items.map((item) => this.renderItem(item)).join("");
  
  // Only add thinking badge during full renders
  const thinkingBadge = (this.state.status === "connecting" || this.state.status === "streaming") 
    ? this.renderThinkingBadge() 
    : "";
  
  this.transcriptEl.innerHTML = html + thinkingBadge;
  this.scrollToBottom();
}
```

## Performance Impact

### Before Fix
- **Full re-renders:** 30-60 per second
- **DOM operations:** 1000s of elements updated per second
- **CPU usage:** High (constant layout thrashing)
- **Visual result:** Severe flickering, unreadable during streaming

### After Fix
- **Full re-renders:** 1-2 per message (only when new items added)
- **DOM operations:** 1 element updated per batch (30 times/sec)
- **CPU usage:** Low (minimal layout recalculation)
- **Visual result:** Smooth typewriter effect, perfectly readable

## Testing

To verify the fix:
1. Send a message that triggers a long response
2. Observe the streaming text
3. **Expected:** Smooth typewriter effect, no flickering
4. **Expected:** Thinking badge glows smoothly at bottom
5. **Expected:** Text is readable throughout streaming

## Related Files

- `web/components/chat-stream/chat-stream.ts` - Main streaming component
- `web/utils/streaming-optimizer.ts` - RAF batching and debouncing
- `docs/STREAMING_DEBOUNCE_FIX.md` - Previous debouncing improvements
- `docs/IMPLEMENTATION_SUMMARY.md` - Overall streaming architecture

## Key Takeaway

**Never re-render the entire transcript during streaming.** Always update only the streaming element directly. Save full re-renders for when the structure changes (new items added).
