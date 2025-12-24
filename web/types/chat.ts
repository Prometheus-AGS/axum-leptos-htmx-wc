/**
 * Chat-related type definitions.
 */

import type { Citation } from "./events";

// ─────────────────────────────────────────────────────────────────────────────
// Chat Roles
// ─────────────────────────────────────────────────────────────────────────────

export type ChatRole = "user" | "assistant" | "tool" | "error" | "system";

// ─────────────────────────────────────────────────────────────────────────────
// Chat Items
// ─────────────────────────────────────────────────────────────────────────────

export interface MessageItem {
  kind: "message";
  role: ChatRole;
  content: string;
  html: string;
}

export interface ThinkingItem {
  kind: "thinking";
  content: string;
  isComplete: boolean;
}

export interface ReasoningItem {
  kind: "reasoning";
  content: string;
  isComplete: boolean;
}

export interface ToolCallItem {
  kind: "tool_call";
  role: "tool";
  callIndex: number;
  id: string;
  name: string;
  argumentsRaw: string;
  status: "streaming" | "complete";
  // Result is added when available
  result?: {
    content: string;
    success: boolean;
  };
}

export interface ToolResultItem {
  kind: "tool_result";
  role: "tool";
  id: string;
  name: string;
  contentRaw: string;
  success: boolean;
}

export interface CitationsItem {
  kind: "citations";
  citations: Citation[];
}

export interface NoticeItem {
  kind: "notice";
  role: "system";
  text: string;
}

export type ChatItem =
  | MessageItem
  | ThinkingItem
  | ReasoningItem
  | ToolCallItem
  | ToolResultItem
  | CitationsItem
  | NoticeItem;

// ─────────────────────────────────────────────────────────────────────────────
// Chat State
// ─────────────────────────────────────────────────────────────────────────────

export type StreamStatus =
  | "idle"
  | "connecting"
  | "streaming"
  | "done"
  | "error";

export interface ChatState {
  requestId: string | null;
  status: StreamStatus;
  items: ChatItem[];
  streamingText: string;
  streamingThinking: string;
  streamingReasoning: string;
  toolCalls: Map<number, ToolCallAccumulator>;
  citations: Citation[];
}

export interface ToolCallAccumulator {
  id?: string;
  name?: string;
  arguments: string;
  status: "streaming" | "complete";
}

// ─────────────────────────────────────────────────────────────────────────────
// Initial State Factory
// ─────────────────────────────────────────────────────────────────────────────

export function createInitialChatState(): ChatState {
  return {
    requestId: null,
    status: "idle",
    items: [],
    streamingText: "",
    streamingThinking: "",
    streamingReasoning: "",
    toolCalls: new Map(),
    citations: [],
  };
}

export function resetChatState(state: ChatState): void {
  state.requestId = null;
  state.status = "idle";
  state.items = [];
  state.streamingText = "";
  state.streamingThinking = "";
  state.streamingReasoning = "";
  state.toolCalls.clear();
  state.citations = [];
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversation Store Types
// ─────────────────────────────────────────────────────────────────────────────

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
