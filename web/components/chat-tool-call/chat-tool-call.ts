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
    return ["name", "call-id", "arguments", "status", "result", "has-result"];
  }

  private _name: string = "";
  private _callId: string = "";
  private _arguments: string = "";
  private _status: "streaming" | "complete" = "streaming";
  private _isExpanded: boolean = false;
  private _result: string = "";
  private _hasResult: boolean = false;

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
    this._result = this.getAttribute("result") ?? "";
    this._hasResult = this.getAttribute("has-result") === "true";
    this.render();
    this.attachEventListeners();
  }

  private attachEventListeners(): void {
    const header = this.querySelector(".tool-call-header");
    if (header) {
      header.addEventListener("click", () => {
        this._isExpanded = !this._isExpanded;
        this.render();
        this.attachEventListeners();
      });
    }
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
      case "result":
        this._result = newValue ?? "";
        break;
      case "has-result":
        this._hasResult = newValue === "true";
        break;
    }

    this.render();
  }

  private render(): void {
    const formattedArgs = formatJsonForDisplay(this._arguments);
    const codeId = createUniqueId("tool-call-code");
    const resultId = createUniqueId("tool-result");

    // Parse result if available
    let resultData: { content: string; success: boolean } | null = null;
    if (this._hasResult && this._result) {
      try {
        resultData = JSON.parse(this._result);
      } catch (e) {
        console.warn("[chat-tool-call] Failed to parse result:", e);
      }
    }

    const statusBadge = resultData
      ? `<span class="text-xs ${resultData.success ? 'bg-success/20 text-success' : 'bg-danger/20 text-danger'} px-2 py-0.5 rounded-full">${resultData.success ? '✓ Success' : '✗ Error'}</span>`
      : this._status === "complete"
      ? '<span class="text-xs bg-success/20 text-success px-2 py-0.5 rounded-full">Complete</span>'
      : '<span class="text-xs bg-warning/20 text-warning px-2 py-0.5 rounded-full animate-pulse">Streaming...</span>';

    const chevronIcon = this._isExpanded
      ? '<path d="M18 15l-6-6-6 6"/>' // ChevronUp
      : '<path d="M6 9l6 6 6-6"/>'; // ChevronDown

    this.innerHTML = `
      <article class="chat-tool-call rounded-xl bg-surfaceContainer overflow-hidden" style="max-width: 800px; width: 100%;">
        <div class="tool-call-header flex items-center justify-between px-4 py-3 bg-surfaceContainerHigh cursor-pointer hover:bg-surfaceContainerHighest transition-colors">
          <div class="flex items-center gap-2 flex-1">
            <svg class="w-4 h-4 text-textPrimary transition-transform ${this._isExpanded ? 'rotate-0' : ''}" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              ${chevronIcon}
            </svg>
            <span class="text-primary">🔧</span>
            <span class="font-medium text-sm text-textPrimary">Tool Call</span>
            <code class="text-xs bg-surfaceContainerHighest text-textPrimary px-2 py-0.5 rounded">${escapeHtml(this._name)}</code>
          </div>
          ${statusBadge}
        </div>
        <div class="tool-call-content transition-all duration-300 ease-in-out ${this._isExpanded ? 'max-h-[2000px] opacity-100' : 'max-h-0 opacity-0 overflow-hidden'}">
          <div class="p-4 space-y-4 bg-surfaceContainer">
            ${
              this._callId
                ? `
              <div class="text-xs text-textSecondary">
                <span class="font-medium">ID:</span> 
                <code class="bg-surfaceContainerHighest text-textPrimary px-1 rounded">${escapeHtml(this._callId)}</code>
              </div>
            `
                : ""
            }
            <div class="bg-surfaceContainerLow rounded-lg p-3">
              <div class="text-xs font-medium text-textSecondary mb-2">Arguments</div>
              <div class="relative">
                <pre id="${codeId}" class="bg-surfaceContainerHighest rounded-lg p-3 text-xs overflow-x-auto max-h-48 text-textPrimary"><code class="language-json">${escapeHtml(formattedArgs)}</code></pre>
                <copy-button target="${codeId}" class="absolute top-2 right-2 opacity-70 hover:opacity-100"></copy-button>
              </div>
            </div>
            ${
              resultData
                ? `
              <div class="bg-surfaceContainerLow rounded-lg p-3">
                <div class="text-xs font-medium text-textSecondary mb-2">Result</div>
                <div class="relative">
                  <pre id="${resultId}" class="bg-surfaceContainerHighest rounded-lg p-3 text-xs overflow-x-auto max-h-64 text-textPrimary"><code>${escapeHtml(resultData.content)}</code></pre>
                  <copy-button target="${resultId}" class="absolute top-2 right-2 opacity-70 hover:opacity-100"></copy-button>
                </div>
              </div>
            `
                : ""
            }
          </div>
        </div>
      </article>
    `;
  }
}
