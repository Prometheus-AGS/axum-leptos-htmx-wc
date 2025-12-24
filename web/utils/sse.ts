/**
 * SSE (Server-Sent Events) connection management.
 */

import type { NormalizedEvent } from "../types/events";
import { parseNormalizedEvent } from "../types/events";

/**
 * Event handler types for SSE connection.
 */
export interface SSEEventHandlers {
  onNormalizedEvent?: (event: NormalizedEvent) => void;
  onAguiEvent?: (event: AguiEvent) => void;
  onOpen?: () => void;
  onError?: (error: Event) => void;
  onClose?: () => void;
}

/**
 * SSE connection options.
 */
export interface SSEConnectionOptions {
  url: string;
  handlers: SSEEventHandlers;
  withCredentials?: boolean;
}

/**
 * SSE connection wrapper with automatic event parsing.
 * @deprecated Use HTMX SSE extension instead.
 */
export class SSEConnection {
  private eventSource: EventSource | null = null;
  private url: string;
  private handlers: SSEEventHandlers;
  private withCredentials: boolean;
  private isConnected = false;

  constructor(options: SSEConnectionOptions) {
    this.url = options.url;
    this.handlers = options.handlers;
    this.withCredentials = options.withCredentials ?? false;
  }

  /**
   * Check if the connection is currently active.
   */
  get connected(): boolean {
    return this.isConnected;
  }

  /**
   * Open the SSE connection.
   */
  connect(): void {
    if (this.eventSource) {
      this.disconnect();
    }

    this.eventSource = new EventSource(this.url, {
      withCredentials: this.withCredentials,
    });

    this.eventSource.onopen = () => {
      this.isConnected = true;
      this.handlers.onOpen?.();
    };

    this.eventSource.onerror = (error) => {
      this.isConnected = false;
      this.handlers.onError?.(error);
    };

    // Bind normalized event handlers
    this.bindNormalizedEvents();

    // Bind AG-UI event handlers
    this.bindAguiEvents();
  }

  /**
   * Close the SSE connection.
   */
  disconnect(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
      this.isConnected = false;
      this.handlers.onClose?.();
    }
  }

  /**
   * Bind handlers for normalized events.
   */
  private bindNormalizedEvents(): void {
    if (!this.eventSource) return;

    const normalizedEventTypes = [
      "stream.start",
      "message.delta",
      "thinking.delta",
      "reasoning.delta",
      "citation.added",
      "memory.update",
      "tool_call.delta",
      "tool_call.complete",
      "tool_result",
      "usage",
      "error",
      "done",
    ];

    for (const eventType of normalizedEventTypes) {
      this.eventSource.addEventListener(eventType, (ev) => {
        const messageEvent = ev as MessageEvent<string>;
        const parsed = parseNormalizedEvent(messageEvent.data);
        if (parsed && this.handlers.onNormalizedEvent) {
          this.handlers.onNormalizedEvent(parsed);
        }
      });
    }
  }

  /**
   * Bind handlers for AG-UI events.
   */
  private bindAguiEvents(): void {
    if (!this.eventSource) return;

    const aguiEventTypes = [
      "agui.stream.start",
      "agui.message.delta",
      "agui.thinking.delta",
      "agui.reasoning.delta",
      "agui.citation.added",
      "agui.memory.update",
      "agui.tool_call.delta",
      "agui.tool_call.complete",
      "agui.tool_result",
      "agui.error",
      "agui.done",
    ];

    for (const eventType of aguiEventTypes) {
      this.eventSource.addEventListener(eventType, (ev) => {
        const messageEvent = ev as MessageEvent<string>;
        const parsed = parseAguiEvent(messageEvent.data);
        if (parsed && this.handlers.onAguiEvent) {
          this.handlers.onAguiEvent(parsed);
        }
      });
    }
  }
}

/**
 * Create a one-shot SSE connection that auto-disconnects on done/error.
 */
export function createOneShotSSE(
  url: string,
  handlers: SSEEventHandlers,
): SSEConnection {
  const wrappedHandlers: SSEEventHandlers = {
    ...handlers,
    onNormalizedEvent: (event) => {
      handlers.onNormalizedEvent?.(event);

      // Auto-close on done or error
      if (event.type === "done" || event.type === "error") {
        connection.disconnect();
      }
    },
  };

  const connection = new SSEConnection({
    url,
    handlers: wrappedHandlers,
  });

  return connection;
}
