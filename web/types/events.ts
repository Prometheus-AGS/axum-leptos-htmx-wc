/**
 * Strongly-typed event definitions for SSE streaming.
 */

// ─────────────────────────────────────────────────────────────────────────────
// Citation Type
// ─────────────────────────────────────────────────────────────────────────────

export interface Citation {
  index: number;
  url: string;
  title?: string;
  snippet?: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Normalized Events (from server)
// ─────────────────────────────────────────────────────────────────────────────

export interface StreamStartEvent {
  type: "stream.start";
  data: {
    request_id: string;
  };
}

export interface MessageDeltaEvent {
  type: "message.delta";
  data: {
    text: string;
  };
}

export interface ThinkingDeltaEvent {
  type: "thinking.delta";
  data: {
    text: string;
  };
}

export interface ReasoningDeltaEvent {
  type: "reasoning.delta";
  data: {
    text: string;
  };
}

export interface CitationAddedEvent {
  type: "citation.added";
  data: Citation;
}

export interface MemoryUpdateEvent {
  type: "memory.update";
  data: {
    key: string;
    value: string;
    operation: "set" | "append" | "delete";
  };
}

export interface ToolCallDeltaEvent {
  type: "tool_call.delta";
  data: {
    call_index: number;        // Match server field name
    id?: string;
    name?: string;
    arguments_delta?: string;  // Match server field name
  };
}

export interface ToolCallCompleteEvent {
  type: "tool_call.complete";
  data: {
    call_index: number;        // Match server field name
    id: string;
    name: string;
    arguments_json: string;    // Match server field name
  };
}

export interface ToolResultEvent {
  type: "tool_result";
  data: {
    id: string;               // Match server field name (was tool_call_id)
    name: string;
    content: string;          // Match server field name (was result)
    success: boolean;
  };
}

export interface UsageEvent {
  type: "usage";
  data: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface ErrorEvent {
  type: "error";
  data: {
    message: string;
    code?: string;
  };
}

export interface DoneEvent {
  type: "done";
  data?: undefined;
}

export type NormalizedEvent =
  | StreamStartEvent
  | MessageDeltaEvent
  | ThinkingDeltaEvent
  | ReasoningDeltaEvent
  | CitationAddedEvent
  | MemoryUpdateEvent
  | ToolCallDeltaEvent
  | ToolCallCompleteEvent
  | ToolResultEvent
  | UsageEvent
  | ErrorEvent
  | DoneEvent;


// ─────────────────────────────────────────────────────────────────────────────
// AG-UI Events (primary protocol)
// ─────────────────────────────────────────────────────────────────────────────

export interface AgUiStreamStartEvent {
  kind: "stream";
  phase: "start";
  request_id: string;
}

export interface AgUiMessageDeltaEvent {
  kind: "message";
  phase: "delta";
  request_id: string;
  delta: {
    text: string;
  };
}

export interface AgUiThinkingDeltaEvent {
  kind: "thinking";
  phase: "delta";
  request_id: string;
  delta: {
    text: string;
  };
}

export interface AgUiReasoningDeltaEvent {
  kind: "reasoning";
  phase: "delta";
  request_id: string;
  delta: {
    text: string;
  };
}

export interface AgUiCitationAddedEvent {
  kind: "citation";
  phase: "added";
  request_id: string;
  citation: Citation;
}

export interface AgUiMemoryUpdateEvent {
  kind: "memory";
  phase: "update";
  request_id: string;
  key: string;
  value: string;
  operation: "set" | "append" | "delete";
}

export interface AgUiToolCallDeltaEvent {
  kind: "tool_call";
  phase: "delta";
  request_id: string;
  call_index: number;
  id?: string;
  name?: string;
  delta: {
    arguments?: string;
  };
}

export interface AgUiToolCallCompleteEvent {
  kind: "tool_call";
  phase: "complete";
  request_id: string;
  call_index: number;
  id: string;
  name: string;
  arguments_json: string;
}

export interface AgUiToolResultEvent {
  kind: "tool_result";
  request_id: string;
  id: string;
  name: string;
  content: string;
  success: boolean;
}

export interface AgUiUsageEvent {
  kind: "usage";
  request_id: string;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface AgUiErrorEvent {
  kind: "error";
  request_id: string;
  message: string;
  code?: string;
}

export interface AgUiDoneEvent {
  kind: "done";
  request_id: string;
}

export type AgUiEvent =
  | AgUiStreamStartEvent
  | AgUiMessageDeltaEvent
  | AgUiThinkingDeltaEvent
  | AgUiReasoningDeltaEvent
  | AgUiCitationAddedEvent
  | AgUiMemoryUpdateEvent
  | AgUiToolCallDeltaEvent
  | AgUiToolCallCompleteEvent
  | AgUiToolResultEvent
  | AgUiUsageEvent
  | AgUiErrorEvent
  | AgUiDoneEvent;

// ─────────────────────────────────────────────────────────────────────────────
// Type Guards
// ─────────────────────────────────────────────────────────────────────────────

export function isNormalizedEvent(obj: unknown): obj is NormalizedEvent {
  return (
    typeof obj === "object" &&
    obj !== null &&
    "type" in obj &&
    typeof (obj as Record<string, unknown>).type === "string"
  );
}

export function isAgUiEvent(obj: unknown): obj is AgUiEvent {
  return (
    typeof obj === "object" &&
    obj !== null &&
    "kind" in obj &&
    typeof (obj as Record<string, unknown>).kind === "string"
  );
}

/**
 * Parse a JSON string into a NormalizedEvent.
 */
export function parseNormalizedEvent(json: string): NormalizedEvent | null {
  try {
    const parsed: unknown = JSON.parse(json);
    if (isNormalizedEvent(parsed)) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Parse a JSON string into an AgUiEvent.
 */
export function parseAgUiEvent(json: string): AgUiEvent | null {
  try {
    const parsed: unknown = JSON.parse(json);
    if (isAgUiEvent(parsed)) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
