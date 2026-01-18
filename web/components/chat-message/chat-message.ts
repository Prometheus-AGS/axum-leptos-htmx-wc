/**
 * Chat Message Web Component
 *
 * Renders a single chat message with markdown support.
 */

import type { ChatRole } from "../../types/chat";
import { createUniqueId, escapeHtml } from "../../utils/html";
import { renderMarkdown } from "../../utils/markdown";

/**
 * Chat Message component for displaying individual messages.
 */
export class ChatMessage extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["role", "content"];
  }

  private _role: ChatRole = "assistant";
  private _content: string = "";

  get role(): ChatRole {
    return this._role;
  }

  set role(value: ChatRole) {
    this._role = value;
    this.render();
  }

  get content(): string {
    return this._content;
  }

  set content(value: string) {
    this._content = value;
    this.render();
  }

  connectedCallback(): void {
    this._role = (this.getAttribute("role") as ChatRole) ?? "assistant";
    this._content = this.getAttribute("content") ?? "";
    this.render();
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue) return;

    if (name === "role") {
      this._role = (newValue as ChatRole) ?? "assistant";
    } else if (name === "content") {
      this._content = newValue ?? "";
    }

    this.render();
  }

  private getRoleConfig(): {
    label: string;
    classes: string;
    icon: string;
  } {
    switch (this._role) {
      case "user":
        return {
          label: "You",
          classes: "bg-bubbleUser text-textPrimary",
          icon: "👤",
        };
      case "assistant":
        return {
          label: "Assistant",
          classes: "bg-bubbleAssistant text-textPrimary",
          icon: "🤖",
        };
      case "tool":
        return {
          label: "Tool",
          classes: "bg-bubbleTool text-textPrimary",
          icon: "🔧",
        };
      case "error":
        return {
          label: "Error",
          classes: "bg-dangerContainer text-danger",
          icon: "❌",
        };
      case "system":
        return {
          label: "System",
          classes: "bg-warningContainer text-warning",
          icon: "⚙️",
        };
      default:
        return {
          label: "Unknown",
          classes: "bg-bubbleAssistant text-textPrimary",
          icon: "💬",
        };
    }
  }

  private render(): void {
    const config = this.getRoleConfig();
    const html =
      this._role === "error"
        ? `<p class="text-danger">${escapeHtml(this._content)}</p>`
        : renderMarkdown(this._content);

    const contentId = createUniqueId("message-content");

    this.innerHTML = `
      <article class="chat-message rounded-xl p-4 ${config.classes} relative group" aria-label="${config.label} message">
        <div class="flex items-center gap-2 text-xs text-textMuted mb-2">
          <span aria-hidden="true">${config.icon}</span>
          <span class="font-medium">${config.label}</span>
        </div>
        <div id="${contentId}" class="prose prose-invert prose-sm max-w-none" data-raw-content="${escapeHtml(this._content)}">
          ${html}
        </div>
        <copy-button target="${contentId}" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity" aria-label="Copy message content"></copy-button>
      </article>
    `;
  }
}
