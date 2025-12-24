import type { AgUiEvent } from "../../types/events";
import type { TranscriptView } from "./transcript-view";
import { renderMarkdown } from "../../utils/markdown";

import { StreamingOptimizer, StreamVelocityTracker, IncrementalMarkdownParser } from "../../utils/streaming-optimizer";
import { createUniqueId } from "../../utils/html";

interface ToolCallAccumulator {
  id: string;
  name: string;
  arguments: string;
}

export class StreamController {
  private view: TranscriptView;
  private streamingOptimizer = new StreamingOptimizer();
  private velocityTracker = new StreamVelocityTracker();
  private markdownParser = new IncrementalMarkdownParser();

  // State
  private _requestId: string | null = null;
  private currentToolCalls = new Map<number, ToolCallAccumulator>();
  private toolIdToDomId = new Map<string, string>(); // real_id -> dom_id
  
  // Buffers
  private textBuffer = "";
  private thinkingBuffer = "";
  private reasoningBuffer = "";

  constructor(view: TranscriptView) {
    this.view = view;
  }

  reset() {
    this._requestId = null;
    this.currentToolCalls.clear();
    this.toolIdToDomId.clear();
    this.textBuffer = "";
    this.thinkingBuffer = "";
    this.reasoningBuffer = "";
    
    this.velocityTracker.reset();
    this.markdownParser.reset();
    this.streamingOptimizer.cancel();
    this.view.reset();
  }

  handleEvent(event: AgUiEvent) {
    // 1. Handle Stream Start
    if (event.kind === "stream" && event.phase === "start") {
        this._requestId = event.request_id;
        return;
    }

    // 2. Handle Deltas
    if (event.kind === "message" && event.phase === "delta") {
        this.handleMessageDelta(event.delta.text);
        return;
    }

    if (event.kind === "thinking" && event.phase === "delta") {
        this.handleThinkingDelta(event.delta.text);
        return;
    }

    if (event.kind === "reasoning" && event.phase === "delta") {
        this.handleReasoningDelta(event.delta.text);
        return;
    }

    // 3. Handle Tool Calls
    if (event.kind === "tool_call" && event.phase === "delta") {
        this.handleToolCallDelta(event);
        return;
    }
    
    if (event.kind === "tool_call" && event.phase === "complete") {
        this.handleToolCallComplete(event);
        return;
    }
    
    if (event.kind === "tool_result") {
        this.handleToolResult(event);
        return;
    }

    // 4. Handle Lifecycle
    if (event.kind === "error") {
        this.handleError(event.message);
        return;
    }

    if (event.kind === "done") {
        this.handleDone();
        return;
    }

    if (event.kind === "usage") {
        this.handleUsage({
            input_tokens: event.prompt_tokens,
            output_tokens: event.completion_tokens,
            // cost and model currently not sent in event
        });
        return;
    }
  }

  private handleMessageDelta(text: string) {
    this.streamingOptimizer.bufferTextChunk(
      "assistant-message", 
      text,
      (flushedChunk) => {
        this.textBuffer += flushedChunk;
        this.flushTextBuffer();
      }
    );
  }

  private handleThinkingDelta(text: string) {
      if (!this.thinkingBuffer) {
          // Initialize thinking block if new
          this.view.upsertItem({
              id: "thinking-block",
              kind: "thinking", 
              content: ""
          });
      }
      this.thinkingBuffer += text;
      
      this.view.upsertItem({
          id: "thinking-block",
          kind: "thinking",
          content: this.thinkingBuffer
      });
  }

  private handleReasoningDelta(text: string) {
      // Similar to thinking but for "reasoning" (chain of thought)
       if (!this.reasoningBuffer) {
          this.view.upsertItem({
              id: "reasoning-block",
              kind: "reasoning", 
              content: ""
          });
      }
      this.reasoningBuffer += text;
      
      this.view.upsertItem({
          id: "reasoning-block",
          kind: "reasoning",
          content: this.reasoningBuffer
      });
  }
  
  private flushTextBuffer() {
      if (!this.textBuffer) return;
      
      // Use incremental parser for smooth Markdown rendering
      const html = this.markdownParser.parse(this.textBuffer, renderMarkdown);
      
      this.view.upsertItem({
          id: "current-message", // Fixed ID for the streaming message
          kind: "message",
          role: "assistant", // Always assistant in this context
          content: this.textBuffer,
          html: html
      });
  }

  private handleToolCallDelta(event: import("../../types/events").AgUiToolCallDeltaEvent) {
    const { call_index, id, name, delta } = event;
    
    // Initialize accumulator if needed
    if (!this.currentToolCalls.has(call_index)) {
      this.currentToolCalls.set(call_index, {
        id: id || "",
        name: name || "",
        arguments: ""
      });
    }

    const accumulator = this.currentToolCalls.get(call_index)!;
    
    // Update fields if present
    if (id) accumulator.id = id;
    if (name) accumulator.name = name;
    if (delta.arguments) accumulator.arguments += delta.arguments;

    // Create a stable DOM ID for this tool call
    if (accumulator.id && !this.toolIdToDomId.has(accumulator.id)) {
        this.toolIdToDomId.set(accumulator.id, createUniqueId());
    }
    
    // Render update
    const domId = this.toolIdToDomId.get(accumulator.id);
    if (domId) {
        this.view.upsertItem({
            id: domId,
            kind: "tool_call",
            content: accumulator.arguments,
            state: "call",
            toolName: accumulator.name
        });
    }
  }

  private handleToolCallComplete(event: import("../../types/events").AgUiToolCallCompleteEvent) {
     // Ensure final state is consistent
     const domId = this.toolIdToDomId.get(event.id);
     if (domId) {
        this.view.upsertItem({
            id: domId,
            kind: "tool_call",
            content: event.arguments_json,
            state: "call",
            toolName: event.name,
            // Could add a "complete" flag here if ViewItem supported it
        });
     }
  }

  private handleToolResult(event: import("../../types/events").AgUiToolResultEvent) {
      // Find the tool call's DOM ID
      // We expect event.tool_call_id or similar. 
      // Note: AgUiToolResultEvent interface in events.ts might need checking if it has call_index or id.
      // Assuming event.id corresponds to the tool call ID we tracked.
      
      const domId = this.toolIdToDomId.get(event.id);
      
      if (domId) {
          this.view.updateToolResult(domId, event.content, !event.success);
      } else {
          // Fallback if we can't link it (e.g. from history load without tracking)
          // But for now, we only handle live aggregation this way.
          console.warn(`[StreamController] Could not find DOM ID for tool result: ${event.id}`);
      }
  }

  private handleError(message: string) {
      this.view.upsertItem({
          id: createUniqueId(),
          kind: "error",
          content: message
      });
  }

  /* New handler for usage events */
  private handleUsage(usage: { input_tokens: number; output_tokens: number; cost?: number; model?: string }) {
      // Dispatch custom event for token counter to pick up
      window.dispatchEvent(new CustomEvent('token-usage-update', { 
          detail: { 
              input: usage.input_tokens, 
              output: usage.output_tokens,
              cost: usage.cost,
              model: usage.model 
          } 
      }));
  }

  private handleDone() {
      // Final flush of optimizer buffers
      this.streamingOptimizer.flushAll();
      
      // Final flush of message buffer (redundant if flushAll calls callback, but safe)
      this.flushTextBuffer();

      // Check if this was the first turn to trigger auto-naming
      // Simple heuristic: if message count in view is small (or we track it via store)
      // Ideally pass a flag or check store. For now, we'll try to trigger it if conversation is new.
      // We can rely on the view to tell us, or just always trigger it non-intrusively.
      // Better: check if title is "New Conversation" before triggering? 
      // This logic is best handled by the ChatStream orchestrator or Store, but triggering here is immediate.
      
      if (this._requestId) {
          // Fire event to let main app handle auto-naming trigger
          window.dispatchEvent(new CustomEvent('stream-completed', { detail: { requestId: this._requestId } }));
      }
  }
}
