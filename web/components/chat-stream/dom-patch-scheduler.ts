/**
 * DOM Patch Scheduler
 * 
 * High-performance batched DOM updates with frame budget management.
 * Ensures smooth 60fps performance during streaming by batching DOM operations
 * and respecting frame time budgets.
 */

export interface DomPatch {
  type: 'append' | 'patch' | 'rekey' | 'remove';
  key: string;
  element?: HTMLElement;
  updates?: Record<string, any>;
  newKey?: string;
  priority?: number; // Higher numbers = higher priority
}

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
  private stats = {
    patchesProcessed: 0,
    framesUsed: 0,
    averageFrameTime: 0
  };

  constructor(config: Partial<SchedulerConfig> = {}) {
    this.config = {
      maxFrameBudgetMs: 12,      // 60fps target with 4ms buffer
      minFrameDelay: 16,         // 60fps = 16.67ms per frame
      batchSize: 10,             // Process up to 10 patches per frame
      ...config
    };
  }

  /**
   * Schedule patches for processing in the next available frame.
   */
  schedule(patches: DomPatch[]): void {
    // Add patches to queue, sorted by priority
    this.patchQueue.push(...patches);
    this.patchQueue.sort((a, b) => (b.priority || 0) - (a.priority || 0));

    // Schedule RAF if not already scheduled
    if (!this.rafId && !this.isProcessing) {
      this.rafId = requestAnimationFrame((timestamp) => {
        this.processBatch(timestamp);
      });
    }
  }

  /**
   * Force immediate processing of all queued patches.
   * Use sparingly - prefer RAF scheduling for smooth performance.
   */
  flush(): void {
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    
    this.isProcessing = true;
    const startTime = performance.now();
    
    while (this.patchQueue.length > 0) {
      const patch = this.patchQueue.shift()!;
      this.applyPatch(patch);
    }
    
    const endTime = performance.now();
    this.updateStats(endTime - startTime);
    this.isProcessing = false;
  }

  /**
   * Cancel all pending patches and RAF callbacks.
   */
  cancel(): void {
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.patchQueue = [];
    this.isProcessing = false;
  }

  /**
   * Get performance statistics.
   */
  getStats() {
    return { ...this.stats };
  }

  /**
   * Process a batch of patches within the frame budget.
   */
  private processBatch(_timestamp: number): void {
    this.isProcessing = true;
    const frameStart = performance.now();
    let patchesProcessed = 0;

    // Process patches until we hit frame budget or batch size limit
    while (
      this.patchQueue.length > 0 && 
      patchesProcessed < this.config.batchSize &&
      (performance.now() - frameStart) < this.config.maxFrameBudgetMs
    ) {
      const patch = this.patchQueue.shift()!;
      this.applyPatch(patch);
      patchesProcessed++;
    }

    const frameTime = performance.now() - frameStart;
    this.updateStats(frameTime);

    // Schedule next frame if there are more patches
    if (this.patchQueue.length > 0) {
      // Add small delay to prevent overwhelming the main thread
      const delay = Math.max(0, this.config.minFrameDelay - frameTime);
      
      if (delay > 0) {
        setTimeout(() => {
          this.rafId = requestAnimationFrame((ts) => this.processBatch(ts));
        }, delay);
      } else {
        this.rafId = requestAnimationFrame((ts) => this.processBatch(ts));
      }
    } else {
      this.rafId = null;
      this.isProcessing = false;
    }
  }

  /**
   * Apply a single DOM patch.
   */
  private applyPatch(patch: DomPatch): void {
    try {
      switch (patch.type) {
        case 'append':
          this.applyAppendPatch(patch);
          break;
        case 'patch':
          this.applyUpdatePatch(patch);
          break;
        case 'rekey':
          this.applyRekeyPatch(patch);
          break;
        case 'remove':
          this.applyRemovePatch(patch);
          break;
        default:
          console.warn('[DomPatchScheduler] Unknown patch type:', patch.type);
      }
    } catch (error) {
      console.error('[DomPatchScheduler] Error applying patch:', patch, error);
    }
  }

  private applyAppendPatch(patch: DomPatch): void {
    if (!patch.element) {
      console.warn('[DomPatchScheduler] Append patch missing element:', patch);
      return;
    }

    // Set the key as a data attribute for tracking
    patch.element.dataset.itemKey = patch.key;
    
    // Find the container (assume parent of existing keyed elements)
    const container = this.findContainer(patch.element);
    if (container) {
      container.appendChild(patch.element);
    }
  }

  private applyUpdatePatch(patch: DomPatch): void {
    const element = this.findElementByKey(patch.key);
    if (!element || !patch.updates) {
      console.warn('[DomPatchScheduler] Update patch missing element or updates:', patch);
      return;
    }

    // Apply updates to element properties and attributes
    for (const [key, value] of Object.entries(patch.updates)) {
      if (key.startsWith('data-')) {
        element.setAttribute(key, String(value));
      } else if (key === 'textContent') {
        element.textContent = String(value);
      } else if (key === 'innerHTML') {
        element.innerHTML = String(value);
      } else if (key === 'className') {
        element.className = String(value);
      } else {
        // Try to set as property first, then as attribute
        try {
          (element as any)[key] = value;
        } catch {
          element.setAttribute(key, String(value));
        }
      }
    }
  }

  private applyRekeyPatch(patch: DomPatch): void {
    const element = this.findElementByKey(patch.key);
    if (!element || !patch.newKey) {
      console.warn('[DomPatchScheduler] Rekey patch missing element or newKey:', patch);
      return;
    }

    // Update the key without moving the DOM node
    element.dataset.itemKey = patch.newKey;
  }

  private applyRemovePatch(patch: DomPatch): void {
    const element = this.findElementByKey(patch.key);
    if (!element) {
      console.warn('[DomPatchScheduler] Remove patch element not found:', patch);
      return;
    }

    element.remove();
  }

  /**
   * Find an element by its item key.
   */
  private findElementByKey(key: string): HTMLElement | null {
    return document.querySelector(`[data-item-key="${key}"]`) as HTMLElement;
  }

  /**
   * Find the container for appending new elements.
   * This is a simple implementation - in practice, you'd pass the container.
   */
  private findContainer(element: HTMLElement): HTMLElement | null {
    // Look for a parent with class 'chat-stream-transcript' or similar
    let current = element.parentElement;
    while (current) {
      if (current.classList.contains('chat-stream-transcript') || 
          current.classList.contains('transcript-container')) {
        return current;
      }
      current = current.parentElement;
    }
    
    // Fallback: look for existing keyed elements and use their parent
    const existingKeyed = document.querySelector('[data-item-key]');
    return existingKeyed?.parentElement as HTMLElement || null;
  }

  /**
   * Update performance statistics.
   */
  private updateStats(frameTime: number): void {
    this.stats.patchesProcessed++;
    this.stats.framesUsed++;
    
    // Calculate rolling average frame time
    const alpha = 0.1; // Smoothing factor
    this.stats.averageFrameTime = 
      this.stats.averageFrameTime * (1 - alpha) + frameTime * alpha;
  }
}

/**
 * Create a DOM patch for appending a new element.
 */
export function createAppendPatch(
  key: string, 
  element: HTMLElement, 
  priority = 0
): DomPatch {
  return {
    type: 'append',
    key,
    element,
    priority
  };
}

/**
 * Create a DOM patch for updating an existing element.
 */
export function createUpdatePatch(
  key: string, 
  updates: Record<string, any>, 
  priority = 0
): DomPatch {
  return {
    type: 'patch',
    key,
    updates,
    priority
  };
}

/**
 * Create a DOM patch for re-keying an element.
 */
export function createRekeyPatch(
  oldKey: string, 
  newKey: string, 
  priority = 0
): DomPatch {
  return {
    type: 'rekey',
    key: oldKey,
    newKey,
    priority
  };
}

/**
 * Create a DOM patch for removing an element.
 */
export function createRemovePatch(key: string, priority = 0): DomPatch {
  return {
    type: 'remove',
    key,
    priority
  };
}