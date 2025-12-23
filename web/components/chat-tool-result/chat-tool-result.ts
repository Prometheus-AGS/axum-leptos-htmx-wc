/**
 * Chat Tool Result Web Component
 *
 * Displays the result of a tool execution.
 */

import { escapeHtml, formatJsonForDisplay, createUniqueId } from "../../utils/html";

/**
 * Chat Tool Result component for displaying tool execution results.
 */
export class ChatToolResult extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["name", "result-id", "content", "success"];
  }

  private _name: string = "";
  private _resultId: string = "";
  private _content: string = "";
  private _success: boolean = true;

  get name(): string {
    return this._name;
  }

  set name(value: string) {
    this._name = value;
    this.render();
  }

  get resultId(): string {
    return this._resultId;
  }

  set resultId(value: string) {
    this._resultId = value;
    this.render();
  }

  get content(): string {
    return this._content;
  }

  set content(value: string) {
    this._content = value;
    this.render();
  }

  get success(): boolean {
    return this._success;
  }

  set success(value: boolean) {
    this._success = value;
    this.render();
  }

  connectedCallback(): void {
    this._name = this.getAttribute("name") ?? "";
    this._resultId = this.getAttribute("result-id") ?? "";
    this._content = this.getAttribute("content") ?? "";
    this._success = this.getAttribute("success") !== "false";
    this.render();
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue) return;

    switch (name) {
      case "name":
        this._name = newValue ?? "";
        break;
      case "result-id":
        this._resultId = newValue ?? "";
        break;
      case "content":
        this._content = newValue ?? "";
        break;
      case "success":
        this._success = newValue !== "false";
        break;
    }

    this.render();
  }

  private render(): void {
    const formattedContent = formatJsonForDisplay(this._content);
    const codeId = createUniqueId("tool-result-code");
    const statusClass = this._success ? "border-success/50" : "border-danger/50";
    const statusIcon = this._success ? "✅" : "❌";
    const statusLabel = this._success ? "Success" : "Failed";

    this.innerHTML = `
      <article class="chat-tool-result rounded-xl border ${statusClass} bg-panel/50 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-2 border-b border-panelBorder bg-panel/50">
          <div class="flex items-center gap-2">
            <span>${statusIcon}</span>
            <span class="font-medium text-sm">Tool Result</span>
            <code class="text-xs bg-codeBg px-2 py-0.5 rounded">${escapeHtml(this._name)}</code>
          </div>
          <span class="text-xs text-textMuted">${statusLabel}</span>
        </div>
        <div class="p-4">
          <div class="relative">
            <pre id="${codeId}" class="bg-codeBg rounded-lg p-3 text-xs overflow-x-auto max-h-64"><code class="language-json">${escapeHtml(formattedContent)}</code></pre>
            <copy-button target="${codeId}" class="absolute top-2 right-2"></copy-button>
          </div>
        </div>
      </article>
    `;
  }
}
