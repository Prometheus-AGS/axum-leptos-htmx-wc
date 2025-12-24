# Streaming Debounce Fix

## Problem

The streaming display was very jagged during message rendering, despite having a `StreamingOptimizer` utility implemented. The optimizer was being called but wasn't actually debouncing properly.

## Root Cause

The `StreamingOptimizer.flushBuffers()` method was flushing **all buffered text** in each RAF frame, which defeated the purpose of debouncing. This caused the UI to update with every single chunk immediately, leading to:

- Jittery text appearance
- Excessive DOM updates
- Poor visual experience
- Wasted rendering cycles

## Solution

### 1. Batch-Based Flushing

Updated `flushBuffers()` to only flush a **batch at a time** instead of all buffered text:

```typescript
// Determine batch size based on stream type
let batchSize = this.config.textBatchSize;
if (streamId.includes('thinking') || streamId.includes('reasoning')) {
  batchSize = this.config.thinkingBatchSize;
} else if (streamId.includes('assistant')) {
  batchSize = this.config.markdownBatchSize;
}

// Only flush up to batch size
const toFlush = bufferedText.slice(0, batchSize);
const remaining = bufferedText.slice(batchSize);
```

### 2. Frame Delay

Added minimum frame delay to smooth out rendering:

```typescript
const timeSinceLastFlush = timestamp - this.lastFlushTime;
const delay = Math.max(0, this.config.minFrameDelay - timeSinceLastFlush);

if (delay > 0) {
  setTimeout(() => {
    this.rafId = requestAnimationFrame((ts) => this.flushBuffers(ts));
  }, delay);
}
```

### 3. Optimized Batch Sizes

Increased batch sizes for smoother appearance:

| Stream Type | Old Size | New Size | Frame Delay |
|-------------|----------|----------|-------------|
| Text        | 16 chars | 20 chars | 16ms (60fps) |
| Markdown    | 32 chars | 40 chars | 16ms (60fps) |
| Thinking    | 50 chars | 60 chars | 20ms |
| Reasoning   | 50 chars | 60 chars | 20ms |

## How It Works Now

1. **Chunk Arrives**: Text chunk is added to buffer
2. **RAF Scheduled**: If not already scheduled, request animation frame
3. **Batch Flush**: On RAF callback, flush only batch-size worth of text
4. **Remaining Buffered**: Keep remaining text in buffer
5. **Next Frame**: Schedule next RAF if more text remains
6. **Frame Pacing**: Ensure minimum 16ms between frames for smooth 60fps

## Benefits

- ✅ **Smooth streaming**: Text appears to "type out" smoothly
- ✅ **Reduced jank**: Fewer DOM updates per second
- ✅ **Better performance**: Respects frame budget (12ms per frame)
- ✅ **Adaptive**: Different batch sizes for different content types
- ✅ **Consistent pacing**: Minimum 16ms between frames ensures smooth 60fps

## Visual Effect

**Before:**
```
H
He
Hel
Hell
Hello
Hello 
Hello w
Hello wo
Hello wor
Hello worl
Hello world  <-- Jittery, updates every chunk
```

**After:**
```
Hello wo
Hello world  <-- Smooth, batched updates
```

## Configuration

The optimizer can be tuned via `StreamingConfig`:

```typescript
const optimizer = new StreamingOptimizer({
  textBatchSize: 20,        // Characters per batch
  markdownBatchSize: 40,    // Markdown tokens per batch
  thinkingBatchSize: 60,    // Thinking block chars per batch
  maxFrameBudgetMs: 12,     // Max ms per frame
  minFrameDelay: 16,        // Min ms between frames (60fps)
});
```

## Files Modified

- `web/utils/streaming-optimizer.ts` - Fixed batching logic and frame pacing
