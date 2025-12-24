/**
 * Database Type Definitions for PGlite
 */

export interface Conversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  is_pinned: boolean;
  server_session_id: string | null;
  message_count: number;
  metadata: Record<string, unknown>;
}

export interface Message {
  id: string;
  conversation_id: string;
  role: "user" | "assistant" | "tool" | "error";
  content: string;
  created_at: string;
  sequence_order: number;
  metadata: Record<string, unknown>;
}

export interface ThinkingBlock {
  id: string;
  conversation_id: string;
  message_id: string | null;
  content: string;
  is_complete: boolean;
  created_at: string;
  sequence_order: number;
}

export interface ReasoningBlock {
  id: string;
  conversation_id: string;
  message_id: string | null;
  content: string;
  is_complete: boolean;
  created_at: string;
  sequence_order: number;
}

export interface ToolCall {
  id: string;
  conversation_id: string;
  message_id: string;
  call_index: number;
  tool_name: string;
  arguments: Record<string, unknown>;
  status: "streaming" | "complete";
  created_at: string;
  sequence_order: number;
}

export interface ToolResult {
  id: string;
  conversation_id: string;
  tool_call_id: string;
  tool_name: string;
  content: string;
  success: boolean;
  created_at: string;
  sequence_order: number;
}

export interface Citation {
  id: string;
  conversation_id: string;
  message_id: string | null;
  url: string;
  title: string | null;
  snippet: string | null;
  citation_index: number;
  created_at: string;
  sequence_order: number;
}

/**
 * Complete conversation turn with all event types
 */
export interface ConversationTurn {
  userMessage: Message | null;
  assistantMessage: Message | null;
  thinkingBlocks: ThinkingBlock[];
  reasoningBlocks: ReasoningBlock[];
  toolCalls: ToolCall[];
  toolResults: ToolResult[];
  citations: Citation[];
}

/**
 * Reconstructed conversation history for rendering
 */
export interface ConversationHistory {
  conversationId: string;
  items: ConversationHistoryItem[];
}

export type ConversationHistoryItem = 
  | { type: 'message'; data: Message }
  | { type: 'thinking'; data: ThinkingBlock }
  | { type: 'reasoning'; data: ReasoningBlock }
  | { type: 'tool_call'; data: ToolCall }
  | { type: 'tool_result'; data: ToolResult }
  | { type: 'citation'; data: Citation };

/**
 * Search result for conversation list
 */
export interface ConversationSearchResult {
  id: string;
  title: string;
  updated_at: string;
  message_count: number;
  is_pinned: boolean;
  preview?: string; // First message preview
}
