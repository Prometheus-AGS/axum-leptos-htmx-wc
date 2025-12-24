/**
 * Conversation Store
 *
 * Manages conversation history with localStorage persistence.
 * Provides CRUD operations for conversations and messages.
 */

export interface ConversationMessage {
  id: string;
  role: "user" | "assistant" | "tool" | "error";
  content: string;
  timestamp: number;
  // For assistant messages with tool calls
  toolCalls?: Array<{
    id: string;
    name: string;
    arguments: string;
  }>;
  // For tool results
  toolResult?: {
    id: string;
    name: string;
    content: string;
    success: boolean;
  };
}

export interface Conversation {
  id: string;
  sessionId: string; // Server session ID
  title: string;
  messages: ConversationMessage[];
  createdAt: number;
  updatedAt: number;
}

export class ConversationStore {
  private storageKey = "chat_conversations";
  private conversations: Map<string, Conversation> = new Map();

  constructor() {
    this.load();
  }

  /**
   * Load all conversations from localStorage.
   */
  private load(): void {
    try {
      const data = localStorage.getItem(this.storageKey);
      if (data) {
        const parsed = JSON.parse(data) as Conversation[];
        this.conversations = new Map(parsed.map((c) => [c.id, c]));
      }
    } catch (error) {
      console.error("[ConversationStore] Failed to load from localStorage:", error);
      this.conversations = new Map();
    }
  }

  /**
   * Save all conversations to localStorage.
   */
  private save(): void {
    try {
      const data = Array.from(this.conversations.values());
      localStorage.setItem(this.storageKey, JSON.stringify(data));
    } catch (error) {
      console.error("[ConversationStore] Failed to save to localStorage:", error);
    }
  }

  /**
   * Load all conversations.
   */
  loadAll(): Conversation[] {
    return Array.from(this.conversations.values()).sort(
      (a, b) => b.updatedAt - a.updatedAt
    );
  }

  /**
   * Get a specific conversation by ID.
   */
  get(id: string): Conversation | null {
    return this.conversations.get(id) ?? null;
  }

  /**
   * Create a new conversation.
   */
  create(sessionId: string): Conversation {
    const now = Date.now();
    const conversation: Conversation = {
      id: this.generateId(),
      sessionId,
      title: "New Conversation",
      messages: [],
      createdAt: now,
      updatedAt: now,
    };

    this.conversations.set(conversation.id, conversation);
    this.save();

    return conversation;
  }

  /**
   * Add a message to a conversation.
   */
  addMessage(conversationId: string, message: ConversationMessage): void {
    const conversation = this.conversations.get(conversationId);
    if (!conversation) {
      console.error(`[ConversationStore] Conversation ${conversationId} not found`);
      return;
    }

    conversation.messages.push(message);
    conversation.updatedAt = Date.now();

    // Auto-generate title from first user message
    if (conversation.title === "New Conversation" && message.role === "user") {
      conversation.title = this.generateTitle(message.content);
    }

    this.save();
  }

  /**
   * Update conversation title.
   */
  updateTitle(conversationId: string, title: string): void {
    const conversation = this.conversations.get(conversationId);
    if (!conversation) {
      console.error(`[ConversationStore] Conversation ${conversationId} not found`);
      return;
    }

    conversation.title = title;
    conversation.updatedAt = Date.now();
    this.save();
  }

  /**
   * Delete a conversation.
   */
  delete(conversationId: string): boolean {
    const deleted = this.conversations.delete(conversationId);
    if (deleted) {
      this.save();
    }
    return deleted;
  }

  /**
   * Generate a unique ID for a conversation.
   */
  private generateId(): string {
    return `conv_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * Generate a title from the first user message.
   */
  private generateTitle(content: string): string {
    // Take first 50 characters, truncate at word boundary
    const maxLength = 50;
    if (content.length <= maxLength) {
      return content;
    }

    const truncated = content.substring(0, maxLength);
    const lastSpace = truncated.lastIndexOf(" ");
    return lastSpace > 0 ? truncated.substring(0, lastSpace) + "..." : truncated + "...";
  }

  /**
   * Clear all conversations (for testing/debugging).
   */
  clear(): void {
    this.conversations.clear();
    localStorage.removeItem(this.storageKey);
  }
}
