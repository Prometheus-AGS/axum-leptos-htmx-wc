import { escapeHtml, createUniqueId } from "../../utils/html";

export type ViewItemKind = "message" | "thinking" | "reasoning" | "tool_call" | "tool_result" | "error" | "citations" | "usage";

export interface ViewItem {
  id: string;
  kind: ViewItemKind;
  role?: "user" | "assistant" | "tool";
  content?: string;
  html?: string;
  name?: string;     // For tool calls
  args?: string;     // For tool calls
  isComplete?: boolean;
  timestamp?: number;
  
  // AG-UI specific fields
  state?: "call" | "result"; // For tool_call/result discrimination
  toolName?: string;         // Explicit tool name
  isError?: boolean;         // For tool results
}

export class TranscriptView {
  private container: HTMLElement;
  private itemMap = new Map<string, HTMLElement>();
  private pendingScroll = false;
  private isUserScrolling = false;

  constructor(container: HTMLElement) {
    this.container = container;
    this.setupScrollListener();
  }

  private setupScrollListener() {
    this.container.addEventListener("scroll", () => {
      const { scrollTop, scrollHeight, clientHeight } = this.container;
      // If user is not at the bottom (with some buffer), they are scrolling up
      this.isUserScrolling = scrollHeight - scrollTop - clientHeight > 100;
    });
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
      } else if (item.content && item.kind !== 'tool_call') {
          // For non-html items (like thinking/reasoning), update text content directly
           // But updateContent handles specific selectors, so we might need specialized update
           if (item.kind === 'thinking' || item.kind === 'reasoning') {
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
    const containerBorder = el.querySelector(".bg-white, .dark\\:bg-gray-900");

    // Reveal result footer
    if (resultContainer) {
        resultContainer.classList.remove("hidden");
        resultContainer.innerHTML = `
            <div class="font-medium mb-1 ${isError ? 'text-red-500' : 'text-green-500'}">${isError ? 'Error' : 'Result'}</div>
            <pre class="overflow-x-auto whitespace-pre-wrap font-mono text-textMuted">${escapeHtml(result)}</pre>
        `;
    }

    // Update status indicator
    if (statusIndicator) {
        statusIndicator.className = `status-indicator w-1.5 h-1.5 rounded-full ${isError ? 'bg-red-500' : 'bg-green-500'}`;
    }

    // Update border color - simplified to just use panelBorder for consistency, or error color if critical
    // Keeping it simple for S-Tier clean look:
    if (containerBorder) {
        if (isError) {
             containerBorder.classList.remove('border-panelBorder');
             containerBorder.classList.add('border-red-500/50');
        }
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
            <div class="relative max-w-[80%] bg-primary text-white rounded-2xl rounded-tr-sm px-4 py-3 shadow-sm">
                <div class="prose prose-invert max-w-none text-sm break-words" id="${uniqueId}">${item.html || ""}</div>
                <div class="absolute -left-10 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <copy-button target="${uniqueId}" text=""></copy-button>
                </div>
            </div>
          `;
        } else {
          el.classList.add("assistant-message", "flex", "gap-3", "group");
          el.innerHTML = `
            <div class="flex-shrink-0 w-8 h-8 rounded-full bg-surfaceContainer flex items-center justify-center text-primary shadow-sm self-end mb-2 border border-panelBorder">
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
            </div>
            <div class="relative max-w-[85%] bg-surfaceContainer rounded-2xl rounded-bl-sm px-4 py-3 shadow-sm">
               <div class="prose max-w-none text-sm text-textPrimary leading-relaxed break-words" id="${uniqueId}">${item.html || ""}</div>
               <div class="absolute -right-10 top-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <copy-button target="${uniqueId}" text=""></copy-button>
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
        
        el.innerHTML = `
           <div class="bg-surface border ${isError ? 'border-red-500/50' : 'border-panelBorder'} rounded-lg shadow-sm overflow-hidden transition-all duration-200">
             <!-- Header -->
             <div class="bg-surfaceVariant px-3 py-2 border-b ${isError ? 'border-red-500/30' : 'border-panelBorder'} flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <div class="status-indicator w-1.5 h-1.5 rounded-full ${isComplete ? (isError ? 'bg-red-500' : 'bg-green-500') : 'bg-blue-500 animate-pulse'}"></div>
                    <code class="text-xs font-semibold text-textPrimary font-mono">${escapeHtml(item.toolName || item.name || "unknown_tool")}</code>
                </div>
                <div class="text-[10px] text-textMuted uppercase tracking-wider font-medium">Tool Call</div>
             </div>
             
             <!-- Body (Arguments) -->
             <div class="p-3 bg-background/50">
                <pre class="tool-args text-xs text-textSecondary font-mono whitespace-pre-wrap overflow-x-auto break-all">${escapeHtml(item.args || "")}</pre>
             </div>

             <!-- Result Footer (Added dynamically via upsert/update) -->
             <div class="tool-result-container hidden border-t border-panelBorder bg-surfaceVariant/50 p-2 text-xs">
                <!-- Result content goes here -->
             </div>
           </div>
        `;
        break;
      }
        
      case "error":
        el.className = "chat-error mb-4 px-4";
        el.innerHTML = `
            <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 px-4 py-3 rounded-lg text-sm flex items-center gap-3">
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
      this.scrollToBottom('instant'); // Always instant when auto-scrolling during stream
      this.pendingScroll = false;
    });
  }

  private scrollToBottom(behavior: ScrollBehavior = 'instant') {
    if (this.isUserScrolling) return;
    
    this.container.scrollTo({
      top: this.container.scrollHeight,
      behavior
    });
  }

  reset() {
      this.container.innerHTML = "";
      this.itemMap.clear();
      this.isUserScrolling = false;
  }
}
