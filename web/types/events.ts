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
    call_index: number;
    id?: string;
    name?: string;
    arguments_delta?: string;
  };
}

export interface ToolCallCompleteEvent {
  type: "tool_call.complete";
  data: {
    call_index: number;
    id: string;
    name: string;
    arguments_json: string;
  };
}

export interface ToolResultEvent {
  type: "tool_result";
  data: {
    id: string;
    name: string;
    content: string;
    success: boolean;
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
  | ErrorEvent
  | DoneEvent;

// ─────────────────────────────────────────────────────────────────────────────
// AG-UI Events (alternative format)
// ─────────────────────────────────────────────────────────────────────────────

export interface AguiMessageDeltaEvent {
  kind: "message";
  phase: "delta";
  request_id: string;
  delta: {
    text: string;
  };
}

export interface AguiThinkingDeltaEvent {
  kind: "thinking";
  phase: "delta";
  request_id: string;
  delta: {
    text: string;
  };
}

export interface AguiToolCallDeltaEvent {
  kind: "tool_call";
  phase: "delta";
  request_id: string;
  call_index: number;
  id?: string | null;
  name?: string | null;
  delta: {
    arguments?: string | null;
  };
}

export interface AguiToolCallCompleteEvent {
  kind: "tool_call";
  phase: "complete";
  request_id: string;
  call_index: number;
  id: string;
  name: string;
  arguments_json: string;
}

export interface AguiToolResultEvent {
  kind: "tool_result";
  request_id: string;
  id: string;
  name: string;
  content: string;
  success: boolean;
}

export interface AguiErrorEvent {
  kind: "error";
  request_id: string;
  message: string;
  code?: string;
}

export interface AguiDoneEvent {
  kind: "done";
  request_id: string;
}

export type AguiEvent =
  | AguiMessageDeltaEvent
  | AguiThinkingDeltaEvent
  | AguiToolCallDeltaEvent
  | AguiToolCallCompleteEvent
  | AguiToolResultEvent
  | AguiErrorEvent
  | AguiDoneEvent;

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

export function isAguiEvent(obj: unknown): obj is AguiEvent {
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
 * Parse a JSON string into an AguiEvent.
 */
export function parseAguiEvent(json: string): AguiEvent | null {
  try {
    const parsed: unknown = JSON.parse(json);
    if (isAguiEvent(parsed)) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
