/**
 * Chat Message Web Component
 *
 * Renders a single chat message with markdown support.
 */

import type { ChatRole } from "../../types/chat";
import { renderMarkdown } from "../../utils/markdown";
import { escapeHtml, createUniqueId } from "../../utils/html";

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
          classes: "bg-primary/10 border-primary/50",
          icon: "👤",
        };
      case "assistant":
        return {
          label: "Assistant",
          classes: "bg-panel border-panelBorder",
          icon: "🤖",
        };
      case "tool":
        return {
          label: "Tool",
          classes: "bg-panel/50 border-panelBorder",
          icon: "🔧",
        };
      case "error":
        return {
          label: "Error",
          classes: "bg-danger/10 border-danger/50",
          icon: "❌",
        };
      case "system":
        return {
          label: "System",
          classes: "bg-warning/10 border-warning/50",
          icon: "⚙️",
        };
      default:
        return {
          label: "Unknown",
          classes: "bg-panel border-panelBorder",
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
      <article class="chat-message rounded-xl border p-4 ${config.classes} relative group">
        <div class="flex items-center gap-2 text-xs text-textMuted mb-2">
          <span>${config.icon}</span>
          <span class="font-medium">${config.label}</span>
        </div>
        <div id="${contentId}" class="prose prose-invert prose-sm max-w-none" data-raw-content="${escapeHtml(this._content)}">
          ${html}
        </div>
        <copy-button target="${contentId}" class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity"></copy-button>
      </article>
    `;
  }
}
