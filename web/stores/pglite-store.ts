/**
 * PGlite Conversation Store
 * 
 * Client-side PostgreSQL database for conversation storage with full event persistence.
 */

import { PGlite } from '@electric-sql/pglite';
import { MIGRATIONS } from './migrations';
import type {
  Conversation,
  Message,
  ThinkingBlock,
  ReasoningBlock,
  ToolCall,
  ToolResult,
  Citation,
  ConversationTurn,
  ConversationHistory,
  ConversationHistoryItem,
  ConversationSearchResult,
} from '../types/database';
import { generateUuid } from '../utils/uuid';

export class PGliteConversationStore {
  private db: PGlite | null = null;
  private sequenceCounter = 0;
  private initPromise: Promise<void> | null = null;

  /**
   * Initialize PGlite database
   */
  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = (async () => {
      console.log('[PGlite] Initializing database...');
      
      // Initialize PGlite with IndexedDB persistence
      // The WASM files should be served from /static/ by the Rust server
      this.db = new PGlite('idb://chat-conversations');
      
      // Run migrations
      await this.runMigrations();
      
      // Migrate from localStorage if needed
      await this.migrateFromLocalStorage();
      
      console.log('[PGlite] Database initialized successfully');
    })();

    return this.initPromise;
  }

  /**
   * Run database migrations
   */
  private async runMigrations(): Promise<void> {
    if (!this.db) throw new Error('Database not initialized');

    // First, ensure schema_migrations table exists
    try {
      await this.db.exec(`
        CREATE TABLE IF NOT EXISTS schema_migrations (
          version INTEGER PRIMARY KEY,
          applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
      `);
    } catch (error) {
      console.error('[PGlite] Failed to create schema_migrations table:', error);
      throw error;
    }

    // Now run migrations
    for (const migration of MIGRATIONS) {
      try {
        // Check if migration already applied
        const result = await this.db.query(
          `SELECT version FROM schema_migrations WHERE version = $1`,
          [migration.version]
        );

        if (result.rows.length === 0) {
          console.log(`[PGlite] Applying migration ${migration.version}: ${migration.name}`);
          await this.db.exec(migration.up);
          console.log(`[PGlite] Migration ${migration.version} applied successfully`);
        } else {
          console.log(`[PGlite] Migration ${migration.version} already applied, skipping`);
        }
      } catch (error) {
        console.error(`[PGlite] Migration ${migration.version} failed:`, error);
        throw error;
      }
    }
  }

  /**
   * Migrate data from localStorage to PGlite
   */
  private async migrateFromLocalStorage(): Promise<void> {
    const migrated = localStorage.getItem('migrated_to_pglite');
    if (migrated) {
      console.log('[PGlite] Already migrated from localStorage');
      return;
    }

    const oldData = localStorage.getItem('chat_conversations');
    if (!oldData) {
      localStorage.setItem('migrated_to_pglite', 'true');
      console.log('[PGlite] No localStorage data to migrate');
      return;
    }

    console.log('[PGlite] Migrating from localStorage...');
    
    try {
      const conversations = JSON.parse(oldData) as Array<{
        id: string;
        title: string;
        sessionId: string;
        messages: Array<{
          id: string;
          role: string;
          content: string;
          timestamp: number;
        }>;
        createdAt: number;
        updatedAt: number;
      }>;

      for (const conv of conversations) {
        // Insert conversation
        await this.db!.query(`
          INSERT INTO conversations (id, title, created_at, updated_at, server_session_id, message_count)
          VALUES ($1, $2, to_timestamp($3 / 1000.0), to_timestamp($4 / 1000.0), $5, $6)
        `, [
          conv.id,
          conv.title,
          conv.createdAt,
          conv.updatedAt,
          conv.sessionId || null,
          conv.messages.length
        ]);

        // Insert messages
        for (let i = 0; i < conv.messages.length; i++) {
          const msg = conv.messages[i];
          await this.db!.query(`
            INSERT INTO messages (id, conversation_id, role, content, created_at, sequence_order)
            VALUES ($1, $2, $3, $4, to_timestamp($5 / 1000.0), $6)
          `, [
            msg.id,
            conv.id,
            msg.role,
            msg.content,
            msg.timestamp,
            i
          ]);
        }
      }

      // Remove old localStorage data
      localStorage.removeItem('chat_conversations');
      localStorage.setItem('migrated_to_pglite', 'true');
      
      console.log(`[PGlite] Migrated ${conversations.length} conversations from localStorage`);
    } catch (error) {
      console.error('[PGlite] Migration from localStorage failed:', error);
      // Don't throw - allow app to continue with empty database
    }
  }

  /**
   * Create a new conversation
   */
  async createConversation(sessionId?: string): Promise<Conversation> {
    if (!this.db) throw new Error('Database not initialized');

    const id = generateUuid();
    const now = new Date().toISOString();

    await this.db.query(`
      INSERT INTO conversations (id, title, created_at, updated_at, server_session_id, message_count)
      VALUES ($1, $2, $3, $4, $5, 0)
    `, [id, 'New Conversation', now, now, sessionId || null]);

    return {
      id,
      title: 'New Conversation',
      created_at: now,
      updated_at: now,
      is_pinned: false,
      server_session_id: sessionId || null,
      message_count: 0,
      metadata: {},
    };
  }

  /**
   * Save a complete conversation turn with all chunks
   */
  async saveConversationTurn(conversationId: string, turn: ConversationTurn): Promise<void> {
    if (!this.db) throw new Error('Database not initialized');

    try {
      // Start transaction
      await this.db.exec('BEGIN');

      // Save user message
      if (turn.userMessage) {
        await this.db.query(`
          INSERT INTO messages (id, conversation_id, role, content, created_at, sequence_order, metadata)
          VALUES ($1, $2, $3, $4, $5, $6, $7)
        `, [
          turn.userMessage.id,
          conversationId,
          turn.userMessage.role,
          turn.userMessage.content,
          turn.userMessage.created_at,
          this.sequenceCounter++,
          JSON.stringify(turn.userMessage.metadata)
        ]);
      }

      // Save assistant message
      if (turn.assistantMessage) {
        const msgId = turn.assistantMessage.id;
        await this.db.query(`
          INSERT INTO messages (id, conversation_id, role, content, created_at, sequence_order, metadata)
          VALUES ($1, $2, $3, $4, $5, $6, $7)
        `, [
          msgId,
          conversationId,
          turn.assistantMessage.role,
          turn.assistantMessage.content,
          turn.assistantMessage.created_at,
          this.sequenceCounter++,
          JSON.stringify(turn.assistantMessage.metadata)
        ]);

        // Save thinking blocks
        for (const thinking of turn.thinkingBlocks) {
          await this.db.query(`
            INSERT INTO thinking_blocks (id, conversation_id, message_id, content, is_complete, created_at, sequence_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
          `, [
            thinking.id,
            conversationId,
            msgId,
            thinking.content,
            thinking.is_complete,
            thinking.created_at,
            this.sequenceCounter++
          ]);
        }

        // Save reasoning blocks
        for (const reasoning of turn.reasoningBlocks) {
          await this.db.query(`
            INSERT INTO reasoning_blocks (id, conversation_id, message_id, content, is_complete, created_at, sequence_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
          `, [
            reasoning.id,
            conversationId,
            msgId,
            reasoning.content,
            reasoning.is_complete,
            reasoning.created_at,
            this.sequenceCounter++
          ]);
        }

        // Save tool calls
        for (const toolCall of turn.toolCalls) {
          await this.db.query(`
            INSERT INTO tool_calls (id, conversation_id, message_id, call_index, tool_name, arguments, status, created_at, sequence_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
          `, [
            toolCall.id,
            conversationId,
            msgId,
            toolCall.call_index,
            toolCall.tool_name,
            JSON.stringify(toolCall.arguments),
            toolCall.status,
            toolCall.created_at,
            this.sequenceCounter++
          ]);
        }

        // Save tool results
        for (const result of turn.toolResults) {
          await this.db.query(`
            INSERT INTO tool_results (id, conversation_id, tool_call_id, tool_name, content, success, created_at, sequence_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
          `, [
            result.id,
            conversationId,
            result.tool_call_id,
            result.tool_name,
            result.content,
            result.success,
            result.created_at,
            this.sequenceCounter++
          ]);
        }

        // Save citations
        for (const citation of turn.citations) {
          await this.db.query(`
            INSERT INTO citations (id, conversation_id, message_id, url, title, citation_index, created_at, sequence_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
          `, [
            citation.id,
            conversationId,
            msgId,
            citation.url,
            citation.title,
            citation.citation_index,
            citation.created_at,
            this.sequenceCounter++
          ]);
        }
      }

      // Update conversation metadata
      await this.db.query(`
        UPDATE conversations 
        SET updated_at = NOW(), 
            message_count = (SELECT COUNT(*) FROM messages WHERE conversation_id = $1)
        WHERE id = $1
      `, [conversationId]);

      // Commit transaction
      await this.db.exec('COMMIT');

      console.log('[PGlite] Saved conversation turn:', {
        conversationId,
        thinking: turn.thinkingBlocks.length,
        reasoning: turn.reasoningBlocks.length,
        toolCalls: turn.toolCalls.length,
        toolResults: turn.toolResults.length,
        citations: turn.citations.length
      });
    } catch (error) {
      await this.db.exec('ROLLBACK');
      console.error('[PGlite] Failed to save conversation turn:', error);
      throw error;
    }
  }

  /**
   * Load complete conversation with all chunks in order
   */
  async loadConversation(conversationId: string): Promise<ConversationHistory> {
    if (!this.db) throw new Error('Database not initialized');

    // Load all data in parallel
    const [messages, thinking, reasoning, toolCalls, toolResults, citations] = await Promise.all([
      this.db.query(`SELECT * FROM messages WHERE conversation_id = $1 ORDER BY sequence_order`, [conversationId]),
      this.db.query(`SELECT * FROM thinking_blocks WHERE conversation_id = $1 ORDER BY sequence_order`, [conversationId]),
      this.db.query(`SELECT * FROM reasoning_blocks WHERE conversation_id = $1 ORDER BY sequence_order`, [conversationId]),
      this.db.query(`SELECT * FROM tool_calls WHERE conversation_id = $1 ORDER BY sequence_order`, [conversationId]),
      this.db.query(`SELECT * FROM tool_results WHERE conversation_id = $1 ORDER BY sequence_order`, [conversationId]),
      this.db.query(`SELECT * FROM citations WHERE conversation_id = $1 ORDER BY sequence_order`, [conversationId])
    ]);

    // Combine all events and sort by sequence_order
    const allEvents: ConversationHistoryItem[] = [
      ...messages.rows.map((m: Message) => ({ type: 'message' as const, data: m })),
      ...thinking.rows.map((t: ThinkingBlock) => ({ type: 'thinking' as const, data: t })),
      ...reasoning.rows.map((r: ReasoningBlock) => ({ type: 'reasoning' as const, data: r })),
      ...toolCalls.rows.map((tc: ToolCall) => ({ type: 'tool_call' as const, data: tc })),
      ...toolResults.rows.map((tr: ToolResult) => ({ type: 'tool_result' as const, data: tr })),
      ...citations.rows.map((c: Citation) => ({ type: 'citation' as const, data: c }))
    ].sort((a, b) => {
      const aOrder = 'sequence_order' in a.data ? a.data.sequence_order : 0;
      const bOrder = 'sequence_order' in b.data ? b.data.sequence_order : 0;
      return aOrder - bOrder;
    });

    return {
      conversationId,
      items: allEvents
    };
  }

  /**
   * Search conversations by text
   */
  async searchConversations(query: string): Promise<ConversationSearchResult[]> {
    if (!this.db) throw new Error('Database not initialized');

    const result = await this.db.query(`
      SELECT DISTINCT
        c.id,
        c.title,
        c.updated_at,
        c.message_count,
        c.is_pinned
      FROM conversations c
      LEFT JOIN messages m ON m.conversation_id = c.id
      WHERE 
        to_tsvector('english', c.title) @@ plainto_tsquery('english', $1)
        OR to_tsvector('english', m.content) @@ plainto_tsquery('english', $1)
      ORDER BY c.is_pinned DESC, c.updated_at DESC
      LIMIT 20
    `, [query]);

    return result.rows as ConversationSearchResult[];
  }

  /**
   * List all conversations
   */
  async listConversations(limit = 50): Promise<ConversationSearchResult[]> {
    if (!this.db) throw new Error('Database not initialized');

    const result = await this.db.query(`
      SELECT id, title, updated_at, message_count, is_pinned
      FROM conversations
      ORDER BY is_pinned DESC, updated_at DESC
      LIMIT $1
    `, [limit]);

    return result.rows as ConversationSearchResult[];
  }

  /**
   * Update conversation title
   */
  async updateTitle(conversationId: string, title: string): Promise<void> {
    if (!this.db) throw new Error('Database not initialized');

    await this.db.query(`
      UPDATE conversations 
      SET title = $1, updated_at = NOW()
      WHERE id = $2
    `, [title, conversationId]);
  }

  /**
   * Toggle conversation pin status
   */
  async togglePin(conversationId: string): Promise<boolean> {
    if (!this.db) throw new Error('Database not initialized');

    const result = await this.db.query(`
      UPDATE conversations 
      SET is_pinned = NOT is_pinned, updated_at = NOW()
      WHERE id = $1
      RETURNING is_pinned
    `, [conversationId]);

    return result.rows[0]?.is_pinned ?? false;
  }

  /**
   * Delete conversation
   */
  async deleteConversation(conversationId: string): Promise<void> {
    if (!this.db) throw new Error('Database not initialized');

    await this.db.query(`DELETE FROM conversations WHERE id = $1`, [conversationId]);
    console.log('[PGlite] Deleted conversation:', conversationId);
  }

  /**
   * Get conversation by ID
   */
  async getConversation(conversationId: string): Promise<Conversation | null> {
    if (!this.db) throw new Error('Database not initialized');

    const result = await this.db.query(`
      SELECT * FROM conversations WHERE id = $1
    `, [conversationId]);

    return result.rows[0] as Conversation || null;
  }

  /**
   * Clear all data (for testing/debugging)
   */
  async clearAll(): Promise<void> {
    if (!this.db) throw new Error('Database not initialized');

    await this.db.exec(`
      DELETE FROM citations;
      DELETE FROM tool_results;
      DELETE FROM tool_calls;
      DELETE FROM reasoning_blocks;
      DELETE FROM thinking_blocks;
      DELETE FROM messages;
      DELETE FROM conversations;
    `);

    console.log('[PGlite] Cleared all data');
  }
}

// Export singleton instance
export const pgliteStore = new PGliteConversationStore();
