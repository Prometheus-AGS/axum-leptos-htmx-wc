/**
 * Chat Mermaid Web Component
 *
 * Renders Mermaid diagrams from code.
 */

import { escapeHtml, createUniqueId } from "../../utils/html";

/**
 * Chat Mermaid component for rendering Mermaid diagrams.
 */
export class ChatMermaid extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["code"];
  }

  private _code: string = "";
  private _rendered: boolean = false;

  get code(): string {
    return this._code;
  }

  set code(value: string) {
    this._code = value;
    this._rendered = false;
    this.render();
  }

  connectedCallback(): void {
    this._code = this.getAttribute("code") ?? this.textContent ?? "";
    this.render();
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue) return;

    if (name === "code") {
      this._code = newValue ?? "";
      this._rendered = false;
    }

    this.render();
  }

  private async render(): Promise<void> {
    const diagramId = createUniqueId("mermaid");

    // Initial render with loading state
    this.innerHTML = `
      <div class="chat-mermaid relative rounded-xl overflow-hidden border border-panelBorder">
        <div class="flex items-center justify-between px-3 py-2 bg-panel border-b border-panelBorder">
          <span class="text-xs text-textMuted bg-panel px-2 py-0.5 rounded">mermaid</span>
          <div class="flex items-center gap-2">
            <button 
              type="button"
              class="text-xs text-textMuted hover:text-textPrimary transition-colors"
              aria-label="Toggle diagram source"
              data-action="toggle-source"
            >
              View Source
            </button>
            <copy-button target="${diagramId}-source"></copy-button>
          </div>
        </div>
        <div id="${diagramId}" class="mermaid-diagram bg-background p-4 flex items-center justify-center min-h-[100px]">
          <span class="text-textMuted">Loading diagram...</span>
        </div>
        <details id="${diagramId}-source-container" class="border-t border-panelBorder">
          <summary class="sr-only">Diagram source</summary>
          <pre id="${diagramId}-source" class="bg-codeBg p-3 text-xs overflow-x-auto"><code>${escapeHtml(this._code)}</code></pre>
        </details>
      </div>
    `;

    // Bind toggle button
    const toggleBtn = this.querySelector('[data-action="toggle-source"]');
    const sourceContainer = this.querySelector(`#${diagramId}-source-container`);

    if (toggleBtn && sourceContainer) {
      toggleBtn.addEventListener("click", () => {
        if (sourceContainer instanceof HTMLDetailsElement) {
          sourceContainer.open = !sourceContainer.open;
          toggleBtn.textContent = sourceContainer.open
            ? "Hide Source"
            : "View Source";
        }
      });
    }

    // Render the diagram
    if (!this._rendered && window.mermaid) {
      try {
        const diagramContainer = this.querySelector(`#${diagramId}`);
        if (diagramContainer) {
          const { svg } = await window.mermaid.render(
            `${diagramId}-svg`,
            this._code,
          );
          diagramContainer.innerHTML = svg;
          this._rendered = true;
        }
      } catch (err) {
        console.error("[chat-mermaid] Failed to render:", err);
        const diagramContainer = this.querySelector(`#${diagramId}`);
        if (diagramContainer) {
          const errorMessage =
            err instanceof Error ? err.message : "Unknown error";
          diagramContainer.innerHTML = `
            <div class="text-danger text-sm p-4">
              <p class="font-medium">Failed to render diagram</p>
              <p class="text-xs text-textMuted mt-1">${escapeHtml(errorMessage)}</p>
            </div>
          `;
        }
      }
    }
  }
}
