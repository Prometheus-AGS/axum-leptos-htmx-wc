/**
 * Streaming Optimizer
 * 
 * Advanced debouncing and smooth rendering for chat streaming UI.
 * Based on production techniques from ChatGPT, Claude, and performance research.
 */

export interface StreamingConfig {
  // Text streaming
  textBatchSize: number;        // Characters per batch (default: 20)
  textBatchDelay: number;       // ms between batches (default: 16ms)
  
  // Markdown streaming  
  markdownBatchSize: number;    // Tokens per batch (default: 40)
  markdownBatchDelay: number;   // ms between batches (default: 16ms)
  
  // Tool calls / structured data
  structuredDelay: number;      // ms to debounce (default: 50ms)
  
  // Thinking / reasoning blocks
  thinkingBatchSize: number;    // Characters per batch (default: 60)
  thinkingBatchDelay: number;   // ms between batches (default: 20ms)
  
  // RAF budget
  maxFrameBudgetMs: number;     // Max ms per frame (default: 12ms for smooth 60fps)
  minFrameDelay: number;        // Min ms between frames (default: 16ms for 60fps)
}

export class StreamingOptimizer {
  public config: StreamingConfig;
  private rafId: number | null = null;
  private buffers: Map<string, string> = new Map();
  private flushCallbacks: Map<string, (text: string) => void> = new Map();
  private lastFlushTime: number = 0;
  
  constructor(config: Partial<StreamingConfig> = {}) {
    this.config = {
      textBatchSize: 50,        // Increased from 20
      textBatchDelay: 32,       // Increased from 16 (30fps instead of 60fps)
      markdownBatchSize: 80,    // Increased from 40
      markdownBatchDelay: 32,   // Increased from 16
      structuredDelay: 50,
      thinkingBatchSize: 100,   // Increased from 60
      thinkingBatchDelay: 32,   // Increased from 20
      maxFrameBudgetMs: 12,
      minFrameDelay: 32,        // Increased from 16 (30fps)
      ...config
    };
  }
  
  /**
   * Buffer text chunk and schedule RAF flush
   */
  bufferTextChunk(streamId: string, chunk: string, onFlush: (text: string) => void): void {
    const current = this.buffers.get(streamId) || '';
    this.buffers.set(streamId, current + chunk);
    this.flushCallbacks.set(streamId, onFlush);
    
    // Schedule RAF if not already scheduled
    if (!this.rafId) {
      this.rafId = requestAnimationFrame((timestamp) => {
        this.flushBuffers(timestamp);
      });
    }
  }
  
  /**
   * Flush buffers respecting frame budget and batch sizes
   */
  private flushBuffers(timestamp: number): void {
    const frameStart = performance.now();
    let hasMoreWork = false;
    
    for (const [streamId, bufferedText] of this.buffers.entries()) {
      // Check frame budget
      if (performance.now() - frameStart > this.config.maxFrameBudgetMs) {
        hasMoreWork = true;
        break;
      }
      
      if (bufferedText.length === 0) continue;
      
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
      
      // Update buffer
      if (remaining.length > 0) {
        this.buffers.set(streamId, remaining);
        hasMoreWork = true;
      } else {
        this.buffers.delete(streamId);
      }
      
      // Flush batch
      const callback = this.flushCallbacks.get(streamId);
      if (callback) {
        callback(toFlush);
      }
    }
    
    // Schedule next frame if there's more work
    if (hasMoreWork) {
      // Add a small delay to smooth out rendering
      const timeSinceLastFlush = timestamp - this.lastFlushTime;
      const delay = Math.max(0, this.config.minFrameDelay - timeSinceLastFlush);
      
      if (delay > 0) {
        setTimeout(() => {
          this.rafId = requestAnimationFrame((ts) => this.flushBuffers(ts));
        }, delay);
      } else {
        this.rafId = requestAnimationFrame((ts) => this.flushBuffers(ts));
      }
    } else {
      this.rafId = null;
      // Clean up callbacks for completed streams
      for (const streamId of this.flushCallbacks.keys()) {
        if (!this.buffers.has(streamId)) {
          this.flushCallbacks.delete(streamId);
        }
      }
    }
    
    this.lastFlushTime = timestamp;
  }
  
  /**
   * Detect stable markdown boundaries to avoid re-parsing
   */
  findStableBoundary(markdown: string): number {
    // Look for complete blocks: paragraphs, code blocks, lists
    const patterns = [
      /\n\n/g,           // Double newline (paragraph break)
      /```[\s\S]*?```/g, // Complete code block
      /\n(?=[#*-])/g,    // Before heading or list
    ];
    
    let lastStable = 0;
    for (const pattern of patterns) {
      const matches = Array.from(markdown.matchAll(pattern));
      if (matches.length > 0) {
        const lastMatch = matches[matches.length - 1];
      if (lastMatch) {
          const matchIndex = lastMatch.index ?? 0;
          lastStable = Math.max(lastStable, matchIndex + lastMatch[0].length);
      }
      }
    }
    
    return lastStable;
  }
  
  /**
   * Adaptive batch sizing based on stream velocity
   */
  calculateAdaptiveBatchSize(streamVelocity: number): number {
    // streamVelocity = characters per second
    if (streamVelocity > 500) {
      // Fast stream: larger batches, less frequent updates
      return 64;
    } else if (streamVelocity > 200) {
      // Medium stream: balanced
      return 32;
    } else {
      // Slow stream: smaller batches, more frequent updates for typing effect
      return 16;
    }
  }
  
  /**
   * Flush all remaining buffers immediately
   */
  flushAll(): void {
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    
    for (const [streamId, bufferedText] of this.buffers.entries()) {
      if (bufferedText.length > 0) {
        const callback = this.flushCallbacks.get(streamId);
        if (callback) {
          callback(bufferedText);
        }
      }
    }
    
    this.buffers.clear();
    this.flushCallbacks.clear();
  }

  /**
   * Cancel all pending flushes
   */
  cancel(): void {
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.buffers.clear();
    this.flushCallbacks.clear();
  }
}

/**
 * Track stream velocity for adaptive batching
 */
export class StreamVelocityTracker {
  private samples: Array<{ chars: number; time: number }> = [];
  private windowMs = 1000; // 1 second window
  
  recordChunk(charCount: number): void {
    const now = performance.now();
    this.samples.push({ chars: charCount, time: now });
    
    // Remove old samples outside window
    this.samples = this.samples.filter(s => now - s.time < this.windowMs);
  }
  
  getVelocity(): number {
    if (this.samples.length === 0) return 0;
    
    const totalChars = this.samples.reduce((sum, s) => sum + s.chars, 0);
    const firstTime = this.samples[0]?.time ?? 0;
    const lastTime = this.samples[this.samples.length - 1]?.time ?? 0;
    const timeSpan = lastTime - firstTime;
    
    return timeSpan > 0 ? (totalChars / timeSpan) * 1000 : 0; // chars per second
  }
  
  reset(): void {
    this.samples = [];
  }
}

/**
 * Incremental Markdown Parser
 * 
 * Only parses new/changed content, caching stable blocks.
 */
export class IncrementalMarkdownParser {
  private stableNodes: Array<{ content: string; html: string }> = [];
  private lastStableBoundary = 0;
  
  /**
   * Parse only the unstable portion of markdown
   */
  parse(fullMarkdown: string, renderFn: (md: string) => string): string {
    // Find new stable boundary
    const newBoundary = this.findStableBoundary(fullMarkdown);
    
    // If boundary hasn't moved, only parse unstable portion
    if (newBoundary === this.lastStableBoundary) {
      const unstableContent = fullMarkdown.slice(newBoundary);
      const unstableHtml = renderFn(unstableContent);
      
      // Combine cached stable HTML with new unstable HTML
      const stableHtml = this.stableNodes.map(n => n.html).join('');
      return stableHtml + unstableHtml;
    }
    
    // Boundary moved - cache new stable content
    const newStableContent = fullMarkdown.slice(this.lastStableBoundary, newBoundary);
    if (newStableContent) {
      const newStableHtml = renderFn(newStableContent);
      this.stableNodes.push({
        content: newStableContent,
        html: newStableHtml
      });
    }
    
    this.lastStableBoundary = newBoundary;
    
    // Parse remaining unstable content
    const unstableContent = fullMarkdown.slice(newBoundary);
    const unstableHtml = renderFn(unstableContent);
    
    const stableHtml = this.stableNodes.map(n => n.html).join('');
    return stableHtml + unstableHtml;
  }
  
  private findStableBoundary(markdown: string): number {
    // Detect complete blocks
    const codeBlockEnd = markdown.lastIndexOf('```\n');
    const paragraphEnd = markdown.lastIndexOf('\n\n');
    const listEnd = markdown.lastIndexOf('\n- ');
    
    return Math.max(0, codeBlockEnd, paragraphEnd, listEnd);
  }
  
  reset(): void {
    this.stableNodes = [];
    this.lastStableBoundary = 0;
  }
}
