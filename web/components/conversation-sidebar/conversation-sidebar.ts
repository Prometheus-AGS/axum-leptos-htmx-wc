/**
 * Conversation Sidebar Component
 * 
 * Collapsible sidebar for conversation management with search, pin, and delete.
 * Based on Sidebar-shadcnui-structure example from docs/htmx.
 */

import { pgliteStore } from '../../stores/pglite-store';
import type { ConversationSearchResult } from '../../types/database';

export class ConversationSidebar extends HTMLElement {
  private isCollapsed = false;
  private searchQuery = '';
  private conversations: ConversationSearchResult[] = [];
  private activeConversationId: string | null = null;

  async connectedCallback(): Promise<void> {
    this.render();
    this.attachEventListeners();
    
    // Wait for PGlite to be initialized before loading conversations
    try {
      await pgliteStore.init();
      await this.loadConversations();
    } catch (error) {
      console.error('[ConversationSidebar] Failed to initialize:', error);
    }
  }

  private render(): void {
    const collapsedClass = this.isCollapsed ? 'collapsed' : '';
    
    this.innerHTML = `
      <aside class="conversation-sidebar ${collapsedClass}" data-collapsed="${this.isCollapsed}">
        <!-- Header -->
        <div class="sidebar-header">
          <button class="collapse-btn" aria-label="${this.isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}">
            <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              ${this.isCollapsed 
                ? '<path d="M9 18l6-6-6-6"/>' // ChevronRight
                : '<path d="M15 18l-6-6 6-6"/>' // ChevronLeft
              }
            </svg>
          </button>
          ${!this.isCollapsed ? `
            <h2 class="sidebar-title">Conversations</h2>
            <button class="new-chat-btn" aria-label="New conversation">
              <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 5v14m-7-7h14"/>
              </svg>
            </button>
          ` : ''}
        </div>

        <!-- Search (only when expanded) -->
        ${!this.isCollapsed ? `
          <div class="sidebar-search">
            <input 
              type="search" 
              class="search-input" 
              placeholder="Search conversations..."
              value="${this.searchQuery}"
            />
            <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
            </svg>
          </div>
        ` : ''}

        <!-- Conversation List -->
        <div class="sidebar-content">
          ${this.isCollapsed ? this.renderCollapsedList() : this.renderExpandedList()}
        </div>
      </aside>
    `;
  }

  private renderExpandedList(): string {
    if (this.conversations.length === 0) {
      return `
        <div class="empty-state">
          <p>No conversations yet</p>
          <p class="empty-hint">Start a new chat to begin</p>
        </div>
      `;
    }

    return `
      <ul class="conversation-list">
        ${this.conversations.map(conv => this.renderConversationItem(conv)).join('')}
      </ul>
    `;
  }

  private renderCollapsedList(): string {
    // Show only icons for recent conversations
    const recent = this.conversations.slice(0, 5);
    
    return `
      <ul class="conversation-list-collapsed">
        ${recent.map(conv => `
          <li class="conversation-item-collapsed ${conv.id === this.activeConversationId ? 'active' : ''}" 
              data-conversation-id="${conv.id}"
              title="${this.escapeHtml(conv.title)}">
            <button class="conversation-btn-collapsed">
              <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              </svg>
            </button>
          </li>
        `).join('')}
      </ul>
    `;
  }

  private renderConversationItem(conv: ConversationSearchResult): string {
    const isActive = conv.id === this.activeConversationId;
    const isPinned = conv.is_pinned;
    
    return `
      <li class="conversation-item ${isActive ? 'active' : ''}" data-conversation-id="${conv.id}">
        <button class="conversation-btn">
          <div class="conversation-content">
            <div class="conversation-title-row">
              ${isPinned ? `
                <svg class="pin-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 9V4h1c.55 0 1-.45 1-1s-.45-1-1-1H7c-.55 0-1 .45-1 1s.45 1 1 1h1v5c0 1.66-1.34 3-3 3v2h5.97v7l1 1 1-1v-7H19v-2c-1.66 0-3-1.34-3-3z"/>
                </svg>
              ` : ''}
              <span class="conversation-title">${this.escapeHtml(conv.title)}</span>
            </div>
            <span class="conversation-meta">${conv.message_count} messages • ${this.formatDate(conv.updated_at)}</span>
          </div>
          <div class="conversation-actions">
            <button class="action-btn pin-btn" data-action="pin" aria-label="${isPinned ? 'Unpin' : 'Pin'} conversation">
              <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M16 9V4h1c.55 0 1-.45 1-1s-.45-1-1-1H7c-.55 0-1 .45-1 1s.45 1 1 1h1v5c0 1.66-1.34 3-3 3v2h5.97v7l1 1 1-1v-7H19v-2c-1.66 0-3-1.34-3-3z"/>
              </svg>
            </button>
            <button class="action-btn delete-btn" data-action="delete" aria-label="Delete conversation">
              <svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </button>
      </li>
    `;
  }

  private attachEventListeners(): void {
    // Collapse button
    this.querySelector('.collapse-btn')?.addEventListener('click', () => {
      this.isCollapsed = !this.isCollapsed;
      this.render();
      this.attachEventListeners();
      this.dispatchEvent(new CustomEvent('sidebar-toggle', { detail: { collapsed: this.isCollapsed } }));
    });

    // New chat button
    this.querySelector('.new-chat-btn')?.addEventListener('click', () => {
      this.dispatchEvent(new CustomEvent('new-conversation'));
    });

    // Search input
    this.querySelector('.search-input')?.addEventListener('input', (e) => {
      this.searchQuery = (e.target as HTMLInputElement).value;
      this.handleSearch();
    });

    // Conversation items
    this.querySelectorAll('.conversation-item, .conversation-item-collapsed').forEach(item => {
      const conversationId = item.getAttribute('data-conversation-id');
      if (!conversationId) return;

      // Click on conversation
      const btn = item.querySelector('.conversation-btn, .conversation-btn-collapsed');
      btn?.addEventListener('click', (e) => {
        // Don't trigger if clicking on action buttons
        if ((e.target as HTMLElement).closest('.action-btn')) return;
        
        this.activeConversationId = conversationId;
        this.dispatchEvent(new CustomEvent('conversation-select', { detail: { conversationId } }));
        this.render();
        this.attachEventListeners();
      });

      // Pin button
      item.querySelector('[data-action="pin"]')?.addEventListener('click', async (e) => {
        e.stopPropagation();
        await this.handlePin(conversationId);
      });

      // Delete button
      item.querySelector('[data-action="delete"]')?.addEventListener('click', async (e) => {
        e.stopPropagation();
        await this.handleDelete(conversationId);
      });
    });
  }

  private async loadConversations(): Promise<void> {
    try {
      this.conversations = await pgliteStore.listConversations(50);
      this.render();
      this.attachEventListeners();
    } catch (error) {
      console.error('[ConversationSidebar] Failed to load conversations:', error);
    }
  }

  private async handleSearch(): Promise<void> {
    try {
      if (this.searchQuery.trim()) {
        this.conversations = await pgliteStore.searchConversations(this.searchQuery);
      } else {
        this.conversations = await pgliteStore.listConversations(50);
      }
      this.render();
      this.attachEventListeners();
    } catch (error) {
      console.error('[ConversationSidebar] Search failed:', error);
    }
  }

  private async handlePin(conversationId: string): Promise<void> {
    try {
      await pgliteStore.togglePin(conversationId);
      await this.loadConversations();
    } catch (error) {
      console.error('[ConversationSidebar] Pin failed:', error);
    }
  }

  private async handleDelete(conversationId: string): Promise<void> {
    const conv = this.conversations.find(c => c.id === conversationId);
    if (!conv) return;

    if (!confirm(`Delete "${conv.title}"? This cannot be undone.`)) {
      return;
    }

    try {
      await pgliteStore.deleteConversation(conversationId);
      
      // If deleted active conversation, notify parent
      if (conversationId === this.activeConversationId) {
        this.activeConversationId = null;
        this.dispatchEvent(new CustomEvent('conversation-deleted', { detail: { conversationId } }));
      }
      
      await this.loadConversations();
    } catch (error) {
      console.error('[ConversationSidebar] Delete failed:', error);
    }
  }

  private formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    
    return date.toLocaleDateString();
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
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
customElements.define('conversation-sidebar', ConversationSidebar);
