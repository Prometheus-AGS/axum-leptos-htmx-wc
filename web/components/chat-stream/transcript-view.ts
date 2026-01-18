import { createUniqueId, escapeHtml } from "../../utils/html";

export type ViewItemKind =
  | "message"
  | "thinking"
  | "reasoning"
  | "tool_call"
  | "tool_result"
  | "error"
  | "citations"
  | "usage";

export interface ViewItem {
  id: string;
  kind: ViewItemKind;
  role?: "user" | "assistant" | "tool";
  content?: string;
  html?: string;
  name?: string; // For tool calls
  args?: string; // For tool calls
  isComplete?: boolean;
  timestamp?: number;

  // AG-UI specific fields
  state?: "call" | "result"; // For tool_call/result discrimination
  toolName?: string; // Explicit tool name
  isError?: boolean; // For tool results
}

export class TranscriptView {
  private container: HTMLElement;
  private itemMap = new Map<string, HTMLElement>();
  private pendingScroll = false;
  private isUserScrolling = false;
  private isAutoScrolling = false;
  private pointerActive = false;
  private lastScrollTop = 0;
  private lastScrollAt = 0;
  private userScrollReleaseTimeout: number | null = null;

  private _handleScroll: EventListener;
  private _handlePointerDown: EventListener;
  private _handlePointerUp: EventListener;
  private _handleWheel: EventListener;

  constructor(container: HTMLElement) {
    this.container = container;
    this._handleScroll = this.handleScroll.bind(this);
    this._handlePointerDown = this.handlePointerDown.bind(this);
    this._handlePointerUp = this.handlePointerUp.bind(this);
    this._handleWheel = this.handleWheel.bind(this);
    this.container.addEventListener("scroll", this._handleScroll, {
      passive: true,
    });
    this.container.addEventListener("pointerdown", this._handlePointerDown, {
      passive: true,
    });
    this.container.addEventListener("pointerup", this._handlePointerUp, {
      passive: true,
    });
    this.container.addEventListener("wheel", this._handleWheel, {
      passive: true,
    });
  }

  private handleScroll() {
    if (this.isAutoScrolling) return;
    const { scrollTop, scrollHeight, clientHeight } = this.container;
    const remaining = scrollHeight - scrollTop - clientHeight;
    const nearBottom = remaining <= 48;

    const now = performance.now();
    const delta = Math.abs(scrollTop - this.lastScrollTop);
    const elapsed = Math.max(1, now - this.lastScrollAt);
    const velocity = delta / elapsed;

    this.lastScrollTop = scrollTop;
    this.lastScrollAt = now;

    if (nearBottom) {
      this.releaseUserScrollLock();
      return;
    }

    if (this.pointerActive || velocity > 0.15) {
      this.lockUserScroll();
    } else {
      this.lockUserScroll();
    }
  }

  private handlePointerDown() {
    this.pointerActive = true;
  }

  private handlePointerUp() {
    this.pointerActive = false;
    this.scheduleScrollRelease();
  }

  private handleWheel() {
    if (!this.isNearBottom()) {
      this.lockUserScroll();
    }
  }

  private isNearBottom() {
    const { scrollTop, scrollHeight, clientHeight } = this.container;
    return scrollHeight - scrollTop - clientHeight <= 48;
  }

  private lockUserScroll() {
    this.isUserScrolling = true;
    this.scheduleScrollRelease();
  }

  private releaseUserScrollLock() {
    this.isUserScrolling = false;
    if (this.userScrollReleaseTimeout !== null) {
      window.clearTimeout(this.userScrollReleaseTimeout);
      this.userScrollReleaseTimeout = null;
    }
  }

  private scheduleScrollRelease() {
    if (this.userScrollReleaseTimeout !== null) {
      window.clearTimeout(this.userScrollReleaseTimeout);
    }
    this.userScrollReleaseTimeout = window.setTimeout(() => {
      if (!this.pointerActive && this.isNearBottom()) {
        this.isUserScrolling = false;
      }
    }, 200);
  }

  destroy() {
    this.container.removeEventListener("scroll", this._handleScroll);
    this.container.removeEventListener("pointerdown", this._handlePointerDown);
    this.container.removeEventListener("pointerup", this._handlePointerUp);
    this.container.removeEventListener("wheel", this._handleWheel);
    this.releaseUserScrollLock();
  }

  /**
   * Append a new item or update it if it exists (idempotent-ish).
   */
  upsertItem(item: ViewItem) {
    let el = this.itemMap.get(item.id);
    if (!el) {
      el = this.createItemElement(item);
      this.container.appendChild(el);
      this.itemMap.set(item.id, el);
      this.scheduleScroll();
    } else {
      // Update content if present
      if (item.html) {
        this.updateContent(item.id, item.html);
      } else if (item.content && item.kind !== "tool_call") {
        // For non-html items (like thinking/reasoning), update text content directly
        // But updateContent handles specific selectors, so we might need specialized update
        if (item.kind === "thinking" || item.kind === "reasoning") {
          this.updateReasoning(item.id, item.content);
        }
      }

      this.updateElementState(el, item);
    }
  }

  updateContent(id: string, html: string) {
    const el = this.itemMap.get(id);
    if (!el) return;

    const contentEl = el.querySelector(".prose, [data-content]");
    if (contentEl) {
      if (contentEl.classList.contains("prose")) {
        contentEl.innerHTML = html;
      } else {
        contentEl.textContent = html; // or innerHTML depending on type
      }
      this.scheduleScroll();
    }
  }

  updateReasoning(id: string, content: string) {
    const el = this.itemMap.get(id);
    if (!el) return;
    const rawContent = el.querySelector("[data-raw-content]");
    if (rawContent) {
      rawContent.textContent = content;
      this.scheduleScroll();
    }
  }

  updateToolArgs(id: string, args: string) {
    const el = this.itemMap.get(id);
    if (!el) return;
    const argsEl = el.querySelector(".tool-args");
    if (argsEl) {
      argsEl.textContent = args;
      this.scheduleScroll();
    }
  }

  completeItem(id: string) {
    const el = this.itemMap.get(id);
    if (el) {
      el.dataset.status = "complete";
      el.classList.remove("streaming");
    }
  }

  updateToolResult(toolId: string, result: string, isError: boolean) {
    const el = this.itemMap.get(toolId);
    if (!el) return;

    const resultContainer = el.querySelector(".tool-result-container");
    const statusIndicator = el.querySelector(".status-indicator");
    const toolCard = el.querySelector(".tool-card");
    const toolHeader = el.querySelector(".tool-card-header");
    const toolBody = el.querySelector(".tool-card-body");

    // Reveal result footer
    if (resultContainer) {
      resultContainer.classList.remove("hidden");
      resultContainer.innerHTML = `
            <div class="font-medium mb-1 ${isError ? "text-danger" : "text-success"}">${isError ? "Error" : "Result"}</div>
            <pre class="overflow-x-auto whitespace-pre-wrap font-mono text-textMuted">${escapeHtml(result)}</pre>
        `;
    }

    // Update status indicator
    if (statusIndicator) {
      statusIndicator.className = `status-indicator w-1.5 h-1.5 rounded-full ${isError ? "bg-danger" : "bg-success"}`;
    }

    if (isError) {
      toolCard?.classList.remove("bg-surfaceContainer");
      toolCard?.classList.add("bg-dangerContainer");
      toolHeader?.classList.remove("bg-surfaceVariant");
      toolHeader?.classList.add("bg-dangerContainer");
      toolBody?.classList.remove("bg-surfaceContainerHighest");
      toolBody?.classList.add("bg-dangerContainer");
      resultContainer?.classList.remove("bg-surfaceVariant");
      resultContainer?.classList.add("bg-dangerContainer");
    }

    // Mark as complete in DOM
    el.dataset.status = "complete";
    this.scheduleScroll();
  }

  private createItemElement(item: ViewItem): HTMLElement {
    const el = document.createElement("div");
    el.id = item.id;
    el.dataset.kind = item.kind;
    el.dataset.status = item.isComplete ? "complete" : "streaming";
    if (item.role) el.dataset.role = item.role;

    // Base classes
    el.className = "chat-item mb-6 fade-in";

    switch (item.kind) {
      case "message": {
        const uniqueId = `msg-${createUniqueId()}`;
        if (item.role === "user") {
          el.classList.add("user-message", "flex", "justify-end", "group");
          el.innerHTML = `
            <div class="relative max-w-[80%] bg-primary text-white rounded-2xl rounded-tr-sm px-4 py-3" role="log" aria-label="User message">
                <div class="prose prose-invert max-w-none text-sm break-words" id="${uniqueId}">${item.html || ""}</div>
                <div class="absolute -left-10 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <copy-button target="${uniqueId}" text="" aria-label="Copy message"></copy-button>
                </div>
            </div>
          `;
        } else {
          el.classList.add("assistant-message", "flex", "gap-3", "group");
          el.innerHTML = `
            <div class="flex-shrink-0 w-8 h-8 rounded-full bg-surfaceContainerHighest flex items-center justify-center text-primary self-end mb-2" aria-hidden="true">
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
            </div>
            <div class="relative max-w-[85%] bg-surfaceContainer rounded-2xl rounded-bl-sm px-4 py-3" role="log" aria-label="Assistant message" aria-live="polite">
               <div class="prose max-w-none text-sm text-textPrimary leading-relaxed break-words" id="${uniqueId}">${item.html || ""}</div>
               <div class="absolute -right-10 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <copy-button target="${uniqueId}" text="" aria-label="Copy message"></copy-button>
               </div>
            </div>
          `;
        }
        break;
      }

      case "tool_call": {
        // Unified tool block: Header (Name+Spinner) | Args (Collapsible) | Result (Footer)
        el.className = "chat-tool-block mb-4 px-4";
        const isComplete = item.isComplete || item.state === "result";
        const isError = item.isError;
        const toolName = item.toolName || item.name || "unknown_tool";

        el.innerHTML = `
           <div class="tool-card bg-surfaceContainer rounded-xl overflow-hidden transition-all duration-200" role="region" aria-label="Tool call: ${toolName}">
             <!-- Header -->
             <div class="tool-card-header bg-surfaceVariant px-3 py-2 flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <div class="status-indicator w-1.5 h-1.5 rounded-full ${isComplete ? (isError ? "bg-danger" : "bg-success") : "bg-info animate-pulse"}" aria-hidden="true"></div>
                    <code class="text-xs font-semibold text-textPrimary font-mono">${escapeHtml(toolName)}</code>
                </div>
                <div class="text-[10px] text-textMuted uppercase tracking-wider font-medium">Tool Call</div>
             </div>
             
             <!-- Body (Arguments) -->
             <div class="tool-card-body p-3 bg-surfaceContainerHighest">
                <pre class="tool-args text-xs text-textSecondary font-mono whitespace-pre-wrap overflow-x-auto break-all" aria-label="Tool arguments">${escapeHtml(item.args || "")}</pre>
             </div>

             <!-- Result Footer (Added dynamically via upsert/update) -->
             <div class="tool-result-container hidden bg-surfaceVariant p-2 text-xs" aria-live="polite">
                <!-- Result content goes here -->
             </div>
           </div>
        `;
        break;
      }

      case "error":
        el.className = "chat-error mb-4 px-4";
        el.innerHTML = `
            <div class="bg-dangerContainer text-danger px-4 py-3 rounded-xl text-sm flex items-center gap-3">
                <svg class="w-5 h-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>
                <span>${escapeHtml(item.content || "Unknown error")}</span>
            </div>
        `;
        break;
    }

    return el;
  }

  private updateElementState(el: HTMLElement, item: ViewItem) {
    if (item.isComplete) {
      el.dataset.status = "complete";
      el.classList.remove("streaming");
      // Hide spinner keys if we had them
      const spinner = el.querySelector(".generic-spinner");
      if (spinner) (spinner as HTMLElement).style.display = "none";
    }
  }

  private scheduleScroll() {
    if (this.pendingScroll) return;
    this.pendingScroll = true;
    requestAnimationFrame(() => {
      this.scrollToBottom("auto"); // Always instant when auto-scrolling during stream
      this.pendingScroll = false;
    });
  }

  private scrollToBottom(behavior: ScrollBehavior = "auto") {
    if (this.isUserScrolling) return;
    this.isAutoScrolling = true;
    this.container.scrollTo({
      top: this.container.scrollHeight,
      behavior,
    });
    requestAnimationFrame(() => {
      this.isAutoScrolling = false;
      if (this.isNearBottom()) {
        this.isUserScrolling = false;
      }
    });
  }

  scrollToBottomSmooth() {
    this.isUserScrolling = false;
    this.scrollToBottom("smooth");
  }

  reset() {
    this.container.innerHTML = "";
    this.itemMap.clear();
    this.isUserScrolling = false;
  }
}
