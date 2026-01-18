/**
 * Conversation Sidebar Component
 *
 * Collapsible sidebar for conversation management with search, pin, and delete.
 * Features date grouping and inline renaming.
 */

import { pgliteStore } from "../../stores/pglite-store";
import type { ConversationSearchResult } from "../../types/database";
import type { ShadcnAlertDialog } from "../ui/shadcn-alert-dialog";
import "../ui/shadcn-alert-dialog";

type DateGroup =
  | "Today"
  | "Yesterday"
  | "Previous 7 Days"
  | "Previous 30 Days"
  | "Older";

export class ConversationSidebar extends HTMLElement {
  private isCollapsed = false;
  private searchQuery = "";
  private conversations: ConversationSearchResult[] = [];
  private activeConversationId: string | null = null;
  private editingId: string | null = null; // ID of conversation being renamed
  private deleteTargetId: string | null = null; // ID of conversation pending deletion
  private _handleDocumentClick: EventListener | null = null;

  async connectedCallback(): Promise<void> {
    this.render();
    this.attachEventListeners();

    // Wait for PGlite to be initialized before loading conversations
    try {
      await pgliteStore.init();
      await this.loadConversations();
    } catch (error) {
      console.error("[ConversationSidebar] Failed to initialize:", error);
    }

    // Listen for outside clicks to cancel editing
    this._handleDocumentClick = this.handleDocumentClick.bind(
      this,
    ) as EventListener;
    document.addEventListener("click", this._handleDocumentClick);
  }

  disconnectedCallback(): void {
    if (this._handleDocumentClick) {
      document.removeEventListener("click", this._handleDocumentClick);
      this._handleDocumentClick = null;
    }
  }

  private handleDocumentClick(e: Event): void {
    if (this.editingId && !(e.target as HTMLElement).closest(".rename-input")) {
      this.editingId = null;
      this.render();
      this.attachEventListeners();
    }
  }

  private render(): void {
    this.innerHTML = `
      <aside class="conversation-sidebar ${this.isCollapsed ? "collapsed" : ""}" data-collapsed="${this.isCollapsed}" role="navigation" aria-label="Conversation history">
        <!-- Header -->
        <div class="sidebar-header">
           ${
             !this.isCollapsed
               ? `
            <div class="flex items-center gap-2 overflow-hidden">
                <span class="sidebar-title text-nowrap">Chats</span>
            </div>
           `
               : ""
           }
           <div class="flex items-center gap-1 ${this.isCollapsed ? "w-full justify-center flex-col gap-3" : ""}">
               ${
                 !this.isCollapsed
                   ? `
                <button class="new-chat-btn text-textMuted" aria-label="New conversation">
                    <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14m-7-7h14"/></svg>
                </button>
               `
                   : ""
               }
               <button class="collapse-btn text-textMuted" aria-label="${this.isCollapsed ? "Expand sidebar" : "Collapse sidebar"}" aria-expanded="${!this.isCollapsed}">
                <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    ${
                      this.isCollapsed
                        ? '<path d="M9 18l6-6-6-6"/>' // ChevronRight
                        : '<path d="M15 18l-6-6 6-6"/>' // ChevronLeft
                    }
                </svg>
               </button>
           </div>
        </div>

        <!-- Search (only when expanded) -->
        ${
          !this.isCollapsed
            ? `
          <div class="sidebar-search">
            <div class="relative">
                <input 
                type="search" 
                class="search-input"
                placeholder="Search..."
                value="${this.searchQuery}"
                aria-label="Search conversations"
                />
                <svg class="search-icon" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
                </svg>
            </div>
          </div>
        `
            : ""
        }

        <!-- Conversation List -->
        <div class="sidebar-content">
          ${this.isCollapsed ? this.renderCollapsedList() : this.renderGroupedList()}
        </div>
        
        <div class="mt-auto p-3 bg-surfaceContainer flex flex-col gap-2">
             <storage-health></storage-health>
             <token-counter 
                input-tokens="0" 
                output-tokens="0" 
                context-limit="128000" 
                model-id="claude-3-5-sonnet-20241022" 
                is-estimate="true"
            ></token-counter>
        </div>
      </aside>

      <!-- Delete Confirmation Alert -->
      <shadcn-alert-dialog id="delete-alert">
        <shadcn-alert-dialog-content>
            <shadcn-alert-dialog-header>
                <shadcn-alert-dialog-title>Delete Conversation?</shadcn-alert-dialog-title>
                <shadcn-alert-dialog-description>
                    This action cannot be undone. This conversation will be permanently deleted from your local database.
                </shadcn-alert-dialog-description>
            </shadcn-alert-dialog-header>
            <shadcn-alert-dialog-footer>
                <shadcn-alert-dialog-cancel>
                    <button class="px-4 py-2 bg-surfaceContainerHighest hover:bg-surfaceContainer text-textPrimary rounded-md text-sm font-medium transition-colors">
                        Cancel
                    </button>
                </shadcn-alert-dialog-cancel>
                <shadcn-alert-dialog-action>
                    <button class="px-4 py-2 bg-danger text-white rounded-md text-sm font-medium transition-colors">
                        Delete
                    </button>
                </shadcn-alert-dialog-action>
            </shadcn-alert-dialog-footer>
        </shadcn-alert-dialog-content>
      </shadcn-alert-dialog>
    `;
  }

  private renderGroupedList(): string {
    if (this.conversations.length === 0) {
      return `
        <div class="flex flex-col items-center justify-center h-48 text-textMuted text-xs">
          <p>No conversations</p>
        </div>
      `;
    }

    const groups: Record<DateGroup, ConversationSearchResult[]> = {
      Today: [],
      Yesterday: [],
      "Previous 7 Days": [],
      "Previous 30 Days": [],
      Older: [],
    };

    // Group conversations
    this.conversations.forEach((conv) => {
      const date = new Date(conv.updated_at);
      const group = this.getDateGroup(date);
      groups[group].push(conv);
    });

    return Object.entries(groups)
      .filter(([_, items]) => items.length > 0)
      .map(
        ([group, items]) => `
        <div class="mb-4" role="group" aria-label="${group}">
            <h3 class="px-2 py-1.5 text-[10px] font-semibold text-textMuted uppercase tracking-wide sticky top-0 bg-surfaceContainer z-10">${group}</h3>
            <ul class="space-y-0.5" role="list">
                ${items.map((conv) => this.renderConversationItem(conv)).join("")}
            </ul>
        </div>
      `,
      )
      .join("");
  }

  private getDateGroup(date: Date): DateGroup {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const lastWeek = new Date(today);
    lastWeek.setDate(lastWeek.getDate() - 7);
    const lastMonth = new Date(today);
    lastMonth.setDate(lastMonth.getDate() - 30);

    const compareDate = new Date(
      date.getFullYear(),
      date.getMonth(),
      date.getDate(),
    );

    if (compareDate.getTime() === today.getTime()) return "Today";
    if (compareDate.getTime() === yesterday.getTime()) return "Yesterday";
    if (compareDate > lastWeek) return "Previous 7 Days";
    if (compareDate > lastMonth) return "Previous 30 Days";
    return "Older";
  }

  private renderCollapsedList(): string {
    // Show only icons for recent conversations
    const recent = this.conversations.slice(0, 5);

    return `
      <ul class="space-y-2 mt-2">
        ${recent
          .map(
            (conv) => `
          <li class="relative group">
            <button 
                class="conversation-btn-collapsed ${conv.id === this.activeConversationId ? "active text-primary" : "text-textMuted"}"
                data-conversation-id="${conv.id}"
                title="${this.escapeHtml(conv.title)}"
            >
              <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
            </button>
          </li>
        `,
          )
          .join("")}
        
        <li class="pt-2 mt-2">
             <button class="new-chat-btn w-10 h-10 mx-auto flex items-center justify-center rounded-lg text-textMuted" aria-label="New conversation">
                <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14m-7-7h14"/></svg>
            </button>
        </li>
      </ul>
    `;
  }

  private renderConversationItem(conv: ConversationSearchResult): string {
    const isActive = conv.id === this.activeConversationId;
    const isEditing = conv.id === this.editingId;

    return `
      <li class="conversation-item ${isActive ? "active" : ""}" data-conversation-id="${conv.id}" role="none">
        ${
          isEditing
            ? `
            <div class="px-2 py-2">
                <input type="text" 
                    class="rename-input w-full text-xs px-2 py-1.5 rounded bg-surfaceContainerHighest text-textPrimary focus:outline-none focus:ring-2 focus:ring-primary/40" 
                    value="${this.escapeHtml(conv.title)}"
                    autofocus
                    aria-label="Rename conversation"
                />
            </div>
        `
            : `
            <button class="conversation-btn w-full text-left" aria-current="${isActive ? "true" : "false"}">
                <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between gap-2 mb-0.5">
                        <span class="text-xs font-medium text-textPrimary truncate block">${this.escapeHtml(conv.title)}</span>
                    </div>
                    <!-- Assuming message_count can proxy for length, theoretically we could store 'last_message_preview' in DB for better UX -->
                    <span class="text-[10px] text-textMuted truncate block">${conv.message_count} messages</span>
                </div>
            </button>

            <!-- Hover Actions -->
            <div class="conversation-actions absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity bg-linear-to-l from-surfaceContainer from-60% to-transparent pl-4 flex items-center gap-1">
                 <button class="action-btn rename-btn p-1 hover:text-primary text-textMuted" data-action="rename" title="Rename" aria-label="Rename conversation">
                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
                 </button>
                 <button class="action-btn delete-btn p-1 hover:text-danger text-textMuted" data-action="delete" title="Delete" aria-label="Delete conversation">
                    <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                 </button>
            </div>
        `
        }
      </li>
    `;
  }

  private attachEventListeners(): void {
    // Collapse button
    this.querySelector(".collapse-btn")?.addEventListener("click", () => {
      this.isCollapsed = !this.isCollapsed;
      this.render();
      this.attachEventListeners();
      this.dispatchEvent(
        new CustomEvent("sidebar-toggle", {
          detail: { collapsed: this.isCollapsed },
        }),
      );
    });

    // New chat button (both locations)
    this.querySelectorAll(".new-chat-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        this.dispatchEvent(new CustomEvent("new-conversation"));
      });
    });

    // Search input
    this.querySelector(".search-input")?.addEventListener("input", (e) => {
      this.searchQuery = (e.target as HTMLInputElement).value;
      this.handleSearch();
    });

    // Keyboard navigation for conversation list
    this.querySelector(".sidebar-content")?.addEventListener(
      "keydown",
      (e: Event) => {
        const keyboardEvent = e as KeyboardEvent;
        if (
          keyboardEvent.key !== "ArrowDown" &&
          keyboardEvent.key !== "ArrowUp"
        )
          return;

        const currentFocus = document.activeElement;
        if (
          !currentFocus ||
          !currentFocus.classList.contains("conversation-btn")
        )
          return;

        const buttons = Array.from(
          this.querySelectorAll(".conversation-btn"),
        ) as HTMLButtonElement[];
        const index = buttons.indexOf(currentFocus as HTMLButtonElement);

        if (keyboardEvent.key === "ArrowDown" && index < buttons.length - 1) {
          buttons[index + 1]?.focus();
          e.preventDefault();
        } else if (keyboardEvent.key === "ArrowUp" && index > 0) {
          buttons[index - 1]?.focus();
          e.preventDefault();
        }
      },
    );

    // Rename Input Logic (Save on blur/enter)
    const renameInput = this.querySelector(".rename-input") as HTMLInputElement;
    if (renameInput) {
      // Focus is auto via attribute, but ensure selection
      renameInput.select();

      const save = async () => {
        if (!this.editingId) return;
        const newTitle = renameInput.value.trim();
        if (newTitle) {
          await pgliteStore.updateTitle(this.editingId, newTitle);
          await this.refresh();
        }
        this.editingId = null;
        this.render();
        this.attachEventListeners();
      };

      renameInput.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          save();
        } else if (e.key === "Escape") {
          this.editingId = null;
          this.render();
          this.attachEventListeners();
        }
      });

      // Use a slight delay or specific check for blur to avoid race conditions with clicks
      renameInput.addEventListener("blur", () => {
        // Optional: save on blur? often annoying if accidental. Let's save.
        save();
      });
    }

    // Conversation items
    this.querySelectorAll(".conversation-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        const li = btn.closest("li");
        const conversationId = li?.getAttribute("data-conversation-id");
        if (conversationId && conversationId !== this.activeConversationId) {
          this.activeConversationId = conversationId;
          this.dispatchEvent(
            new CustomEvent("conversation-changed", {
              detail: { conversationId },
              bubbles: true,
              composed: true,
            }),
          );
          this.render();
          this.attachEventListeners();
        }
      });
    });

    // Collapsed buttons
    this.querySelectorAll("[data-conversation-id] > button").forEach((btn) => {
      if (btn.classList.contains("conversation-btn")) return; // handled above
      btn.addEventListener("click", () => {
        const conversationId = btn.parentElement?.getAttribute(
          "data-conversation-id",
        );
        if (conversationId) {
          this.activeConversationId = conversationId;
          this.dispatchEvent(
            new CustomEvent("conversation-changed", {
              detail: { conversationId },
              bubbles: true,
              composed: true,
            }),
          );
        }
      });
    });

    // Action Buttons
    this.querySelectorAll("[data-action]").forEach((btn) => {
      btn.addEventListener("click", async (e) => {
        e.stopPropagation();
        const li = btn.closest("li");
        const conversationId = li?.getAttribute("data-conversation-id");
        if (!conversationId) return;

        const action = btn.getAttribute("data-action");
        if (action === "rename") {
          this.editingId = conversationId;
          this.render();
          this.attachEventListeners();
        } else if (action === "delete") {
          this.deleteTargetId = conversationId;
          const alert = this.querySelector(
            "#delete-alert",
          ) as ShadcnAlertDialog;
          if (alert) alert.open();
        }
      });
    });

    // Alert Dialog Actions
    const alert = this.querySelector("#delete-alert") as ShadcnAlertDialog;
    if (alert) {
      alert.addEventListener("shadcn-alert-action-click", async () => {
        if (this.deleteTargetId) {
          await this.handleDelete(this.deleteTargetId);
          this.deleteTargetId = null;
          alert.close();
        }
      });

      alert.addEventListener("shadcn-alert-close-click", () => {
        this.deleteTargetId = null;
        alert.close();
      });
    }
  }

  private async loadConversations(): Promise<void> {
    try {
      this.conversations = await pgliteStore.listConversations(50);
      this.render();
      this.attachEventListeners();
    } catch (error) {
      console.error(
        "[ConversationSidebar] Failed to load conversations:",
        error,
      );
    }
  }

  private async handleSearch(): Promise<void> {
    try {
      if (this.searchQuery.trim()) {
        this.conversations = await pgliteStore.searchConversations(
          this.searchQuery,
        );
      } else {
        this.conversations = await pgliteStore.listConversations(50);
      }
      this.render();
      this.attachEventListeners();
    } catch (error) {
      console.error("[ConversationSidebar] Search failed:", error);
    }
  }

  private async handleDelete(conversationId: string): Promise<void> {
    const conv = this.conversations.find((c) => c.id === conversationId);
    if (!conv) return;

    // Confirmed via Alert Dialog

    try {
      await pgliteStore.deleteConversation(conversationId);

      // If deleted active conversation, notify parent
      if (conversationId === this.activeConversationId) {
        this.activeConversationId = null;
        this.dispatchEvent(
          new CustomEvent("conversation-deleted", {
            detail: { conversationId },
          }),
        );
      }

      await this.loadConversations();
    } catch (error) {
      console.error("[ConversationSidebar] Delete failed:", error);
    }
  }

  private escapeHtml(text: string): string {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * Public API: Set active conversation
   */
  setActiveConversation(conversationId: string | null): void {
    this.activeConversationId = conversationId;
    this.render();
    this.attachEventListeners();
  }

  /**
   * Public API: Refresh conversation list
   */
  async refresh(): Promise<void> {
    await this.loadConversations();
  }
}

// Register the custom element
customElements.define("conversation-sidebar", ConversationSidebar);
