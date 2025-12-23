/**
 * Chat Stream Web Component
 *
 * Manages SSE connection and renders chat messages, tool calls, and results.
 */

import type { NormalizedEvent } from "../../types/events";
import type {
  ChatState,
  ChatItem,
  ToolCallAccumulator,
} from "../../types/chat";
import { createInitialChatState, resetChatState } from "../../types/chat";
import { SSEConnection } from "../../utils/sse";
import { renderMarkdown } from "../../utils/markdown";
import { escapeHtml, formatJsonForDisplay, createUniqueId } from "../../utils/html";

/**
 * Chat Stream component that handles SSE streaming and renders chat content.
 */
export class ChatStream extends HTMLElement {
  private state: ChatState = createInitialChatState();
  private connection: SSEConnection | null = null;
  private transcriptEl: HTMLElement | null = null;
  private statusEl: HTMLElement | null = null;

  static get observedAttributes(): string[] {
    return ["stream-url", "session-id"];
  }

  get streamUrl(): string {
    return this.getAttribute("stream-url") ?? "/stream";
  }

  get sessionId(): string {
    return this.getAttribute("session-id") ?? "";
  }

  connectedCallback(): void {
    this.render();
    this.setupElements();
  }

  disconnectedCallback(): void {
    this.disconnect();
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue !== newValue) {
      // Re-render if attributes change
      if (name === "stream-url" || name === "session-id") {
        this.disconnect();
      }
    }
  }

  /**
   * Start the SSE stream.
   * Can be called from HTMX response handler.
   */
  startStream(responseJson?: string): void {
    // Parse response if provided (from HTMX)
    if (responseJson) {
      try {
        const response = JSON.parse(responseJson) as {
          session_id?: string;
          stream_url?: string;
        };
        if (response.stream_url) {
          this.setAttribute("stream-url", response.stream_url);
        }
      } catch {
        console.warn("[chat-stream] Failed to parse response JSON");
      }
    }

    this.connect();
  }

  /**
   * Connect to the SSE stream.
   */
  connect(): void {
    this.disconnect();
    resetChatState(this.state);
    this.state.status = "connecting";
    this.updateStatus("Connecting...");
    this.renderTranscript();

    this.connection = new SSEConnection({
      url: this.streamUrl,
      handlers: {
        onOpen: () => {
          this.state.status = "streaming";
          this.updateStatus("Streaming...");
        },
        onNormalizedEvent: (event) => this.handleEvent(event),
        onError: () => {
          this.state.status = "error";
          this.updateStatus("Connection error");
        },
        onClose: () => {
          if (this.state.status === "streaming") {
            this.state.status = "done";
            this.updateStatus("Done");
          }
        },
      },
    });

    this.connection.connect();
  }

  /**
   * Disconnect from the SSE stream.
   */
  disconnect(): void {
    if (this.connection) {
      this.connection.disconnect();
      this.connection = null;
    }
  }

  /**
   * Handle a normalized event from the SSE stream.
   */
  private handleEvent(event: NormalizedEvent): void {
    switch (event.type) {
      case "stream.start":
        this.state.requestId = event.data.request_id;
        break;

      case "message.delta":
        this.handleMessageDelta(event.data.text);
        break;

      case "thinking.delta":
        this.handleThinkingDelta(event.data.text);
        break;

      case "reasoning.delta":
        this.handleReasoningDelta(event.data.text);
        break;

      case "citation.added":
        this.state.citations.push(event.data);
        break;

      case "tool_call.delta":
        this.handleToolCallDelta(event.data);
        break;

      case "tool_call.complete":
        this.handleToolCallComplete(event.data);
        break;

      case "tool_result":
        this.handleToolResult(event.data);
        break;

      case "error":
        this.handleError(event.data);
        break;

      case "done":
        this.handleDone();
        break;
    }

    this.renderTranscript();
  }

  /**
   * Handle message delta events.
   */
  private handleMessageDelta(text: string): void {
    this.state.streamingText += text;

    // Update or create assistant message item
    const lastItem = this.state.items[this.state.items.length - 1];
    if (lastItem?.kind === "message" && lastItem.role === "assistant") {
      lastItem.content = this.state.streamingText;
      lastItem.html = renderMarkdown(this.state.streamingText);
    } else {
      this.state.items.push({
        kind: "message",
        role: "assistant",
        content: this.state.streamingText,
        html: renderMarkdown(this.state.streamingText),
      });
    }
  }

  /**
   * Handle thinking delta events.
   */
  private handleThinkingDelta(text: string): void {
    this.state.streamingThinking += text;

    // Find or create thinking item
    let thinkingItem = this.state.items.find(
      (item) => item.kind === "thinking" && !item.isComplete,
    );

    if (!thinkingItem) {
      thinkingItem = {
        kind: "thinking",
        content: "",
        isComplete: false,
      };
      this.state.items.push(thinkingItem);
    }

    if (thinkingItem.kind === "thinking") {
      thinkingItem.content = this.state.streamingThinking;
    }
  }

  /**
   * Handle reasoning delta events.
   */
  private handleReasoningDelta(text: string): void {
    this.state.streamingReasoning += text;

    // Find or create reasoning item
    let reasoningItem = this.state.items.find(
      (item) => item.kind === "reasoning" && !item.isComplete,
    );

    if (!reasoningItem) {
      reasoningItem = {
        kind: "reasoning",
        content: "",
        isComplete: false,
      };
      this.state.items.push(reasoningItem);
    }

    if (reasoningItem.kind === "reasoning") {
      reasoningItem.content = this.state.streamingReasoning;
    }
  }

  /**
   * Handle tool call delta events.
   */
  private handleToolCallDelta(data: {
    call_index: number;
    id?: string;
    name?: string;
    arguments_delta?: string;
  }): void {
    const acc: ToolCallAccumulator = this.state.toolCalls.get(
      data.call_index,
    ) ?? {
      arguments: "",
      status: "streaming",
    };

    if (data.id) acc.id = data.id;
    if (data.name) acc.name = data.name;
    if (data.arguments_delta) acc.arguments += data.arguments_delta;

    this.state.toolCalls.set(data.call_index, acc);
    this.upsertToolCallItem(data.call_index, acc);
  }

  /**
   * Handle tool call complete events.
   */
  private handleToolCallComplete(data: {
    call_index: number;
    id: string;
    name: string;
    arguments_json: string;
  }): void {
    const acc: ToolCallAccumulator = {
      id: data.id,
      name: data.name,
      arguments: data.arguments_json,
      status: "complete",
    };

    this.state.toolCalls.set(data.call_index, acc);
    this.upsertToolCallItem(data.call_index, acc);
  }

  /**
   * Handle tool result events.
   */
  private handleToolResult(data: {
    id: string;
    name: string;
    content: string;
    success: boolean;
  }): void {
    this.state.items.push({
      kind: "tool_result",
      role: "tool",
      id: data.id,
      name: data.name,
      contentRaw: data.content,
      success: data.success,
    });
  }

  /**
   * Handle error events.
   */
  private handleError(data: { message: string }): void {
    this.state.status = "error";
    this.state.items.push({
      kind: "message",
      role: "error",
      content: data.message,
      html: `<p class="text-danger">${escapeHtml(data.message)}</p>`,
    });
    this.updateStatus("Error");
    this.disconnect();
  }

  /**
   * Handle done event.
   */
  private handleDone(): void {
    this.state.status = "done";

    // Mark thinking/reasoning as complete
    for (const item of this.state.items) {
      if (item.kind === "thinking" || item.kind === "reasoning") {
        item.isComplete = true;
      }
    }

    // Add citations if any
    if (this.state.citations.length > 0) {
      this.state.items.push({
        kind: "citations",
        citations: [...this.state.citations],
      });
    }

    this.updateStatus("Done");
    this.disconnect();
  }

  /**
   * Update or insert a tool call item.
   */
  private upsertToolCallItem(
    callIndex: number,
    acc: ToolCallAccumulator,
  ): void {
    const existingIndex = this.state.items.findIndex(
      (item) => item.kind === "tool_call" && item.callIndex === callIndex,
    );

    const item = {
      kind: "tool_call" as const,
      role: "tool" as const,
      callIndex,
      id: acc.id ?? "",
      name: acc.name ?? "unknown",
      argumentsRaw: acc.arguments,
      status: acc.status,
    };

    if (existingIndex >= 0) {
      this.state.items[existingIndex] = item;
    } else {
      this.state.items.push(item);
    }
  }

  /**
   * Initial render.
   */
  private render(): void {
    this.innerHTML = `
      <div class="chat-stream-container h-full flex flex-col">
        <div class="chat-stream-status text-xs text-textMuted px-4 py-1 border-b border-panelBorder">
          Ready
        </div>
        <div class="chat-stream-transcript flex-1 overflow-y-auto px-4 py-4 space-y-4">
        </div>
      </div>
    `;
  }

  /**
   * Setup element references.
   */
  private setupElements(): void {
    this.transcriptEl = this.querySelector(".chat-stream-transcript");
    this.statusEl = this.querySelector(".chat-stream-status");
  }

  /**
   * Update the status display.
   */
  private updateStatus(status: string): void {
    if (this.statusEl) {
      this.statusEl.textContent = status;
    }

    // Also update the header status if it exists
    const headerStatus = document.querySelector("#chat-status");
    if (headerStatus) {
      headerStatus.textContent = status;
    }
  }

  /**
   * Render the transcript.
   */
  private renderTranscript(): void {
    if (!this.transcriptEl) return;

    const html = this.state.items.map((item) => this.renderItem(item)).join("");
    this.transcriptEl.innerHTML = html;

    // Scroll to bottom
    this.transcriptEl.scrollTop = this.transcriptEl.scrollHeight;

    // Trigger mermaid rendering for any diagrams
    this.renderMermaidDiagrams();
  }

  /**
   * Render a single chat item.
   */
  private renderItem(item: ChatItem): string {
    switch (item.kind) {
      case "message":
        return this.renderMessage(item);
      case "thinking":
        return this.renderThinking(item);
      case "reasoning":
        return this.renderReasoning(item);
      case "tool_call":
        return this.renderToolCall(item);
      case "tool_result":
        return this.renderToolResult(item);
      case "citations":
        return this.renderCitations(item);
      case "notice":
        return `<div class="notice text-sm text-textMuted">${escapeHtml(item.text)}</div>`;
      default:
        return "";
    }
  }

  /**
   * Render a message item.
   */
  private renderMessage(item: {
    role: string;
    html: string;
  }): string {
    const roleClass =
      item.role === "assistant"
        ? "bg-panel border-panelBorder"
        : item.role === "error"
          ? "bg-danger/10 border-danger"
          : "bg-primary/10 border-primary";

    const roleLabel =
      item.role === "assistant"
        ? "Assistant"
        : item.role === "error"
          ? "Error"
          : "You";

    return `
      <article class="chat-message rounded-xl border p-4 ${roleClass}">
        <div class="text-xs text-textMuted mb-2 font-medium">${roleLabel}</div>
        <div class="prose prose-invert prose-sm max-w-none">${item.html}</div>
      </article>
    `;
  }

  /**
   * Render thinking item.
   */
  private renderThinking(item: {
    content: string;
    isComplete: boolean;
  }): string {
    const statusBadge = item.isComplete
      ? '<span class="text-xs bg-success/20 text-success px-2 py-0.5 rounded-full">Complete</span>'
      : '<span class="text-xs bg-warning/20 text-warning px-2 py-0.5 rounded-full animate-pulse">Thinking...</span>';

    return `
      <details class="chat-thinking rounded-xl border border-panelBorder bg-panel/50" ${item.isComplete ? "" : "open"}>
        <summary class="px-4 py-2 cursor-pointer flex items-center justify-between">
          <span class="text-sm font-medium">💭 Thinking</span>
          ${statusBadge}
        </summary>
        <div class="px-4 pb-4 text-sm text-textMuted whitespace-pre-wrap">${escapeHtml(item.content)}</div>
      </details>
    `;
  }

  /**
   * Render reasoning item.
   */
  private renderReasoning(item: {
    content: string;
    isComplete: boolean;
  }): string {
    const statusBadge = item.isComplete
      ? '<span class="text-xs bg-success/20 text-success px-2 py-0.5 rounded-full">Complete</span>'
      : '<span class="text-xs bg-primary/20 text-primary px-2 py-0.5 rounded-full animate-pulse">Reasoning...</span>';

    return `
      <details class="chat-reasoning rounded-xl border border-panelBorder bg-panel/50" ${item.isComplete ? "" : "open"}>
        <summary class="px-4 py-2 cursor-pointer flex items-center justify-between">
          <span class="text-sm font-medium">🧠 Reasoning</span>
          ${statusBadge}
        </summary>
        <div class="px-4 pb-4 text-sm text-textMuted whitespace-pre-wrap">${escapeHtml(item.content)}</div>
      </details>
    `;
  }

  /**
   * Render a tool call item.
   */
  private renderToolCall(item: {
    name: string;
    id: string;
    argumentsRaw: string;
    status: "streaming" | "complete";
  }): string {
    const statusBadge =
      item.status === "complete"
        ? '<span class="text-xs bg-success/20 text-success px-2 py-0.5 rounded-full">Complete</span>'
        : '<span class="text-xs bg-warning/20 text-warning px-2 py-0.5 rounded-full animate-pulse">Streaming...</span>';

    const formattedArgs = formatJsonForDisplay(item.argumentsRaw);
    const codeId = createUniqueId("code");

    return `
      <article class="chat-tool-call rounded-xl border border-panelBorder bg-panel/50 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-2 border-b border-panelBorder bg-panel/50">
          <div class="flex items-center gap-2">
            <span class="text-primary">🔧</span>
            <span class="font-medium text-sm">Tool Call</span>
            <code class="text-xs bg-codeBg px-2 py-0.5 rounded">${escapeHtml(item.name)}</code>
          </div>
          ${statusBadge}
        </div>
        <div class="p-4 space-y-2">
          <div class="text-xs text-textMuted">
            <span class="font-medium">ID:</span> <code class="bg-codeBg px-1 rounded">${escapeHtml(item.id)}</code>
          </div>
          <div class="relative">
            <pre id="${codeId}" class="bg-codeBg rounded-lg p-3 text-xs overflow-x-auto"><code class="language-json">${escapeHtml(formattedArgs)}</code></pre>
            <copy-button target="${codeId}" class="absolute top-2 right-2"></copy-button>
          </div>
        </div>
      </article>
    `;
  }

  /**
   * Render a tool result item.
   */
  private renderToolResult(item: {
    name: string;
    id: string;
    contentRaw: string;
    success: boolean;
  }): string {
    const formattedContent = formatJsonForDisplay(item.contentRaw);
    const codeId = createUniqueId("code");
    const statusClass = item.success ? "border-success/50" : "border-danger/50";
    const statusIcon = item.success ? "✅" : "❌";

    return `
      <article class="chat-tool-result rounded-xl border ${statusClass} bg-panel/50 overflow-hidden">
        <div class="flex items-center justify-between px-4 py-2 border-b border-panelBorder bg-panel/50">
          <div class="flex items-center gap-2">
            <span>${statusIcon}</span>
            <span class="font-medium text-sm">Tool Result</span>
            <code class="text-xs bg-codeBg px-2 py-0.5 rounded">${escapeHtml(item.name)}</code>
          </div>
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

  /**
   * Render citations.
   */
  private renderCitations(item: {
    citations: Array<{ url: string; title?: string; index: number }>;
  }): string {
    const citationItems = item.citations
      .map(
        (c) => `
        <li class="flex items-start gap-2">
          <span class="text-primary font-medium">[${c.index + 1}]</span>
          <a href="${escapeHtml(c.url)}" target="_blank" rel="noopener noreferrer" 
             class="text-primary hover:underline truncate">
            ${escapeHtml(c.title ?? c.url)}
          </a>
        </li>
      `,
      )
      .join("");

    return `
      <article class="chat-citations rounded-xl border border-panelBorder bg-panel/50 p-4">
        <div class="text-sm font-medium mb-2">📚 Sources</div>
        <ul class="space-y-1 text-sm">${citationItems}</ul>
      </article>
    `;
  }

  /**
   * Render any mermaid diagrams in the transcript.
   */
  private async renderMermaidDiagrams(): Promise<void> {
    const mermaidBlocks = this.querySelectorAll(".language-mermaid");
    if (mermaidBlocks.length > 0 && window.mermaid) {
      try {
        await window.mermaid.run({
          nodes: mermaidBlocks as NodeListOf<HTMLElement>,
        });
      } catch (err) {
        console.warn("[chat-stream] Mermaid rendering failed:", err);
      }
    }
  }
}
