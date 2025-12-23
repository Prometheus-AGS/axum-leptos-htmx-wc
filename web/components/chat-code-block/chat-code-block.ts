/**
 * Chat Code Block Web Component
 *
 * Displays syntax-highlighted code with copy functionality.
 */

import { escapeHtml, createUniqueId } from "../../utils/html";
import { highlightCode } from "../../utils/markdown";

/**
 * Chat Code Block component for displaying code with syntax highlighting.
 */
export class ChatCodeBlock extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["language", "code"];
  }

  private _language: string = "";
  private _code: string = "";

  get language(): string {
    return this._language;
  }

  set language(value: string) {
    this._language = value;
    this.render();
  }

  get code(): string {
    return this._code;
  }

  set code(value: string) {
    this._code = value;
    this.render();
  }

  connectedCallback(): void {
    this._language = this.getAttribute("language") ?? "";
    this._code = this.getAttribute("code") ?? this.textContent ?? "";
    this.render();
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue) return;

    if (name === "language") {
      this._language = newValue ?? "";
    } else if (name === "code") {
      this._code = newValue ?? "";
    }

    this.render();
  }

  private render(): void {
    const codeId = createUniqueId("code-block");
    const highlighted = this._language
      ? highlightCode(this._code, this._language)
      : escapeHtml(this._code);

    const languageBadge = this._language
      ? `<span class="text-xs text-textMuted bg-panel px-2 py-0.5 rounded">${escapeHtml(this._language)}</span>`
      : "";

    this.innerHTML = `
      <div class="chat-code-block relative rounded-xl overflow-hidden border border-panelBorder">
        <div class="flex items-center justify-between px-3 py-2 bg-panel border-b border-panelBorder">
          ${languageBadge}
          <copy-button target="${codeId}"></copy-button>
        </div>
        <pre id="${codeId}" class="bg-codeBg p-4 overflow-x-auto text-sm"><code class="hljs language-${escapeHtml(this._language)}">${highlighted}</code></pre>
      </div>
    `;
  }
}
