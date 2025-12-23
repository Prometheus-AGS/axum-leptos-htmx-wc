/**
 * Chat Tool Call Web Component
 *
 * Displays a tool call with its arguments.
 */

import { escapeHtml, formatJsonForDisplay, createUniqueId } from "../../utils/html";

/**
 * Chat Tool Call component for displaying tool invocations.
 */
export class ChatToolCall extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["name", "call-id", "arguments", "status"];
  }

  private _name: string = "";
  private _callId: string = "";
  private _arguments: string = "";
  private _status: "streaming" | "complete" = "streaming";

  get name(): string {
    return this._name;
  }

  set name(value: string) {
    this._name = value;
    this.render();
  }

  get callId(): string {
    return this._callId;
  }

  set callId(value: string) {
    this._callId = value;
    this.render();
  }

  get arguments(): string {
    return this._arguments;
  }

  set arguments(value: string) {
    this._arguments = value;
    this.render();
  }

  get status(): "streaming" | "complete" {
    return this._status;
  }

  set status(value: "streaming" | "complete") {
    this._status = value;
    this.render();
  }

  connectedCallback(): void {
    this._name = this.getAttribute("name") ?? "";
    this._callId = this.getAttribute("call-id") ?? "";
    this._arguments = this.getAttribute("arguments") ?? "";
    this._status =
      (this.getAttribute("status") as "streaming" | "complete") ?? "streaming";
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
      case "call-id":
        this._callId = newValue ?? "";
        break;
      case "arguments":
        this._arguments = newValue ?? "";
        break;
      case "status":
        this._status = (newValue as "streaming" | "complete") ?? "streaming";
        break;
    }

    this.render();
  }

  private render(): void {
    const formattedArgs = formatJsonForDisplay(this._arguments);
    const codeId = createUniqueId("tool-call-code");

    const statusBadge =
      this._status === "complete"
        ? '<span class="text-xs bg-success/20 text-success px-2 py-0.5 rounded-full">Complete</span>'
        : '<span class="text-xs bg-warning/20 text-warning px-2 py-0.5 rounded-full animate-pulse">Streaming...</span>';

    this.innerHTML = `
      <article class="chat-tool-call rounded-xl border border-panelBorder bg-panel/50 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-2 border-b border-panelBorder bg-panel/50">
          <div class="flex items-center gap-2">
            <span class="text-primary">🔧</span>
            <span class="font-medium text-sm">Tool Call</span>
            <code class="text-xs bg-codeBg px-2 py-0.5 rounded">${escapeHtml(this._name)}</code>
          </div>
          ${statusBadge}
        </div>
        <div class="p-4 space-y-2">
          ${
            this._callId
              ? `
            <div class="text-xs text-textMuted">
              <span class="font-medium">ID:</span> 
              <code class="bg-codeBg px-1 rounded">${escapeHtml(this._callId)}</code>
            </div>
          `
              : ""
          }
          <div class="relative">
            <pre id="${codeId}" class="bg-codeBg rounded-lg p-3 text-xs overflow-x-auto"><code class="language-json">${escapeHtml(formattedArgs)}</code></pre>
            <copy-button target="${codeId}" class="absolute top-2 right-2"></copy-button>
          </div>
        </div>
      </article>
    `;
  }
}
