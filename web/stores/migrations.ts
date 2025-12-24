/**
 * PGlite Database Migrations
 * 
 * Schema for storing conversations with full event persistence including
 * thinking blocks, reasoning blocks, tool calls, tool results, and citations.
 */

export const MIGRATIONS = [
  {
    version: 1,
    name: 'initial_schema',
    up: `
      -- Core conversations table
      CREATE TABLE IF NOT EXISTS conversations (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        is_pinned BOOLEAN DEFAULT FALSE,
        server_session_id TEXT,
        message_count INTEGER DEFAULT 0,
        metadata JSONB DEFAULT '{}'::jsonb
      );

      CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at DESC);
      CREATE INDEX IF NOT EXISTS idx_conversations_pinned ON conversations(is_pinned) WHERE is_pinned = TRUE;

      -- Messages table with sequence ordering
      CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
        role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'tool', 'error')),
        content TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        sequence_order INTEGER NOT NULL,
        metadata JSONB DEFAULT '{}'::jsonb
      );

      CREATE INDEX IF NOT EXISTS idx_messages_conversation_seq ON messages(conversation_id, sequence_order);

      -- Thinking blocks (for reasoning models)
      CREATE TABLE IF NOT EXISTS thinking_blocks (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
        message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
        content TEXT NOT NULL,
        is_complete BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        sequence_order INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_thinking_conversation_seq ON thinking_blocks(conversation_id, sequence_order);

      -- Reasoning blocks (for reasoning models)
      CREATE TABLE IF NOT EXISTS reasoning_blocks (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
        message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
        content TEXT NOT NULL,
        is_complete BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        sequence_order INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_reasoning_conversation_seq ON reasoning_blocks(conversation_id, sequence_order);

      -- Tool calls table
      CREATE TABLE IF NOT EXISTS tool_calls (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
        message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
        call_index INTEGER NOT NULL,
        tool_name TEXT NOT NULL,
        arguments JSONB NOT NULL,
        status TEXT CHECK (status IN ('streaming', 'complete')) DEFAULT 'complete',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        sequence_order INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_tool_calls_conversation_seq ON tool_calls(conversation_id, sequence_order);

      -- Tool results table
      CREATE TABLE IF NOT EXISTS tool_results (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
        tool_call_id TEXT NOT NULL REFERENCES tool_calls(id) ON DELETE CASCADE,
        tool_name TEXT NOT NULL,
        content TEXT NOT NULL,
        success BOOLEAN NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        sequence_order INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_tool_results_conversation_seq ON tool_results(conversation_id, sequence_order);

      -- Citations table
      CREATE TABLE IF NOT EXISTS citations (
        id TEXT PRIMARY KEY,
        conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
        message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
        url TEXT NOT NULL,
        title TEXT,
        citation_index INTEGER NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        sequence_order INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_citations_conversation_seq ON citations(conversation_id, sequence_order);

      -- Full-text search indexes
      CREATE INDEX IF NOT EXISTS idx_conversations_title_search 
        ON conversations USING gin(to_tsvector('english', title));
      CREATE INDEX IF NOT EXISTS idx_messages_content_search 
        ON messages USING gin(to_tsvector('english', content));
      CREATE INDEX IF NOT EXISTS idx_thinking_content_search 
        ON thinking_blocks USING gin(to_tsvector('english', content));
      CREATE INDEX IF NOT EXISTS idx_reasoning_content_search 
        ON reasoning_blocks USING gin(to_tsvector('english', content));

      -- Migration tracking
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version INTEGER PRIMARY KEY,
        applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );

      INSERT INTO schema_migrations (version) VALUES (1);
    `,
  },
];
