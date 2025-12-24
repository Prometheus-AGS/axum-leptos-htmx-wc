/**
 * Chat Stream Web Component (Enhanced with HTMX SSE & Keyed DOM)
 *
 * Manages SSE connection via HTMX, processes events with StreamController,
 * manages view with TranscriptView, and persists all events to PGlite database.
 */

import type { AgUiEvent } from "../../types/events";
import { parseAgUiEvent } from "../../types/events";
import { renderMarkdown } from "../../utils/markdown";
import { createUniqueId } from "../../utils/html";
import { generateUuid } from "../../utils/uuid";
import { pgliteStore } from "../../stores/pglite-store";
import type { ConversationTurn, Message } from "../../types/database";

import { TranscriptView } from "./transcript-view";
import { StreamController } from "./stream-controller";

// Extend Window interface for Alpine
declare global {
  interface Window {
    Alpine: any;
  }
}

/**
 * Chat Stream component with HTMX SSE transport and keyed DOM updates.
 */
export class ChatStream extends HTMLElement {
  // Logic & View
  private view: TranscriptView | null = null;
  private controller: StreamController | null = null;
  
  // Bound Event Handlers for proper cleanup
  private _handleConversationChanged: EventListener | null = null;
  private _handleStreamCompleted: EventListener | null = null;
    
  private debugMode = false;
  // private sseContainer: HTMLElement | null = null;

  // View state is now in TranscriptView
  private conversationId: string | null = null;
  private currentTurn: ConversationTurn = this.createEmptyTurn();
  private sequenceOrder = 0;
  
  // Persistence Accumulators
  private accumulatedText = "";
  private accumulatedThinking = "";
  private accumulatedReasoning = "";

  static get observedAttributes(): string[] {
    return ["stream-url", "session-id"];
  }

  get streamUrl(): string {
    return this.getAttribute("stream-url") ?? "/stream";
  }

  get sessionId(): string {
    return this.getAttribute("session-id") ?? "";
  }

  connectedCallback(): void {
    // 1. Setup DOM structure
    this.innerHTML = `
      <div id="chat-transcript" class="flex flex-col h-full overflow-y-auto px-4 py-4 space-y-6 scroll-smooth"></div>
      <div id="sse-listener" style="display:none;"></div>
    `;
    
    const transcriptEl = this.querySelector("#chat-transcript") as HTMLElement;
    // this.sseContainer = this.querySelector("#sse-listener") as HTMLElement;

    // 2. Initialize Controller & View
    this.view = new TranscriptView(transcriptEl);
    this.controller = new StreamController(this.view);

    // 3. Setup HTMX SSE Listener
    // No longer using HTMX for SSE - managed natively by EventSource in startStream
    // WE KEEP sseContainer for backwards compat with any CSS or structure relying on it,
    // but we don't attach listeners.
    
    // 4. Other Listeners
    this._handleConversationChanged = this.handleConversationChanged.bind(this) as unknown as EventListener;
    this._handleStreamCompleted = this.handleStreamCompleted.bind(this) as unknown as EventListener;
    
    window.addEventListener('conversation-changed', this._handleConversationChanged);
    window.addEventListener('stream-completed', this._handleStreamCompleted);
    
    // Check for debug mode active
    this.debugMode = new URLSearchParams(window.location.search).has('debug') || localStorage.getItem('debug') === 'true';
  }

  disconnectedCallback(): void {
    if (this.controller) this.controller.reset();
    if (this.view) this.view.reset();
    
    // Clean up HTMX attributes to ensure connection closes
    // Clean up EventSource
    this.closeStream();

    this.closeStream();

    if (this._handleConversationChanged) {
        window.removeEventListener('conversation-changed', this._handleConversationChanged);
        this._handleConversationChanged = null;
    }
    if (this._handleStreamCompleted) {
        window.removeEventListener('stream-completed', this._handleStreamCompleted);
        this._handleStreamCompleted = null;
    }
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue !== newValue) {
      if (name === "stream-url" || name === "session-id") {
        // If actively streaming, update logic if needed
      }
    }
  }

  private eventSource: EventSource | null = null;
  
  /**
   * Handle stream completion to trigger auto-naming
   */
  private async handleStreamCompleted(_e: CustomEvent) {
       if (!this.conversationId) return;

       // Check if we need to auto-name (only for new/untitled conversations)
       const conv = await pgliteStore.getConversation(this.conversationId);
       if (!conv || (conv.title !== "New Conversation" && conv.title !== "Untitled")) {
           return;
       }
       
       // Only auto-name if we have a user message
       let userMessageText = "";
       let assistantMessageText = "";

       if (this.currentTurn.userMessage) {
           userMessageText = this.currentTurn.userMessage.content;
           if (this.currentTurn.assistantMessage) {
               assistantMessageText = this.currentTurn.assistantMessage.content;
           }
       } else {
           // Fallback to fetching history if currentTurn is empty (e.g. reload)
           const history = await pgliteStore.loadConversation(this.conversationId);
           const firstUserMsg = history.items.find(item => item.type === 'message' && item.data.role === 'user');
           const firstAssistantMsg = history.items.find(item => item.type === 'message' && item.data.role === 'assistant');
           
           if (firstUserMsg && firstUserMsg.type === 'message') {
               userMessageText = firstUserMsg.data.content;
           }
           if (firstAssistantMsg && firstAssistantMsg.type === 'message') {
               assistantMessageText = firstAssistantMsg.data.content;
           }
       }
       
       if (!userMessageText) return;

       try {
           console.log("[chat-stream] Auto-generating title...");
           const response = await fetch('/api/generate-title', {
               method: 'POST',
               headers: { 'Content-Type': 'application/json' },
               body: JSON.stringify({ message: userMessageText, assistant_message: assistantMessageText })
           });
           
           if (response.ok) {
               const data = await response.json();
               if (data.title) {
                   await pgliteStore.updateTitle(this.conversationId, data.title);
                   // Dispatch event to update sidebar
                   window.dispatchEvent(new CustomEvent('conversation-updated', { 
                       detail: { conversationId: this.conversationId } 
                   }));
                   console.log("[chat-stream] Title updated:", data.title);
               }
           }
       } catch (err) {
           console.error("[chat-stream] Failed to auto-generate title:", err);
       }
  }

  /**
   * Start the SSE stream using native EventSource.
   */
  startStream(responseJson?: string): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    let url = this.streamUrl;

    if (responseJson) {
      try {
        const response = JSON.parse(responseJson) as { session_id?: string; stream_url?: string };
        if (response.stream_url) {
          url = response.stream_url;
          this.setAttribute("stream-url", url);
        }
      } catch (e) {
        console.warn("[chat-stream] Failed to parse response JSON", e);
      }
    }

    this.prepareNewStreamState();

    console.log("[chat-stream] Connecting to SSE:", url);
    this.eventSource = new EventSource(url);

    // List of all AG-UI event names
    const eventTypes = [
      "agui.stream.start",
      "agui.message.delta",
      "agui.thinking.delta",
      "agui.reasoning.delta",
      "agui.citation.added",
      "agui.memory.update",
      "agui.tool_call.delta",
      "agui.tool_call.complete",
      "agui.tool_result",
      "agui.usage",
      "agui.error",
      "agui.done"
    ];

    // Add listeners for all event types
    eventTypes.forEach(type => {
      this.eventSource?.addEventListener(type, (e: MessageEvent) => {
        this.handleSseMessage(type, e.data);
      });
    });

    // Handle generic errors
    this.eventSource.onerror = (e) => {
        console.error("[chat-stream] SSE Error:", e);
        this.view?.upsertItem({
            id: createUniqueId(),
            kind: "error",
            content: "Connection interrupted."
        });
        this.closeStream();
    };

    this.eventSource.onopen = () => {
        console.log("[chat-stream] SSE Connected");
    }
  }

  private closeStream() {
      if (this.eventSource) {
          this.eventSource.close();
          this.eventSource = null;
          console.log("[chat-stream] SSE Closed");
          this.saveTurnForPersistence();
      }
  }

  private prepareNewStreamState() {
     this.controller?.reset();
     this.accumulatedText = "";
     this.accumulatedThinking = "";
     this.accumulatedReasoning = "";
     this.currentTurn = this.createEmptyTurn();
  }

  /**
   * Handle incoming SSE Message.
   */
  private handleSseMessage(type: string, rawData: string): void {
    if (this.debugMode && type !== "agui.message.delta") {
       console.log("[chat-stream] Received SSE:", type, rawData);
    }

    // Manual parsing
    let agUiEvent: AgUiEvent | null = null;
    try {
        agUiEvent = parseAgUiEvent(rawData);
    } catch (e) {
        console.error("[chat-stream] Failed to parse SSE data:", e);
    }

    if (!agUiEvent) {
        console.warn("[chat-stream] Dropping unknown/unparsable event:", type);
        return;
    }

    // 1. Pass to Controller for View Updates
    this.controller?.handleEvent(agUiEvent);

    // 2. Accumulate for Persistence
    this.accumulateForPersistence(agUiEvent);

    if (agUiEvent.kind === "done") {
        this.closeStream();
    }
  }

  // ---------------------------------------------------------------------------
  // Persistence Logic
  // ---------------------------------------------------------------------------

  private accumulateForPersistence(event: AgUiEvent): void {
    switch (event.kind) {
      case "message":
        if (event.phase === "delta") {
            this.accumulatedText += event.delta.text;
        }
        break;
      case "thinking":
        if (event.phase === "delta") {
            this.accumulatedThinking += event.delta.text;
        }
        break;
      case "reasoning":
        if (event.phase === "delta") {
            this.accumulatedReasoning += event.delta.text;
        }
        break;
      case "tool_call":
        if (event.phase === "complete" && this.conversationId) {
             this.currentTurn.toolCalls.push({
                id: event.id,
                conversation_id: this.conversationId,
                message_id: "",
                call_index: event.call_index,
                tool_name: event.name,
                arguments: JSON.parse(event.arguments_json || "{}"),
                status: "complete",
                created_at: new Date().toISOString(),
                sequence_order: this.sequenceOrder++
             });
        }
        break;
      case "tool_result":
        if (this.conversationId) {
             this.currentTurn.toolResults.push({
                id: generateUuid(),
                conversation_id: this.conversationId,
                tool_call_id: event.id,
                tool_name: event.name,
                content: event.content,
                success: event.success,
                created_at: new Date().toISOString(),
                sequence_order: this.sequenceOrder++
             });
        }
        break;
      case "citation":
          this.currentTurn.citations.push({
              id: generateUuid(),
              conversation_id: this.conversationId || "",
              message_id: this.currentTurn.assistantMessage?.id || null,
              url: event.citation.url,
              title: event.citation.title || null,
              snippet: event.citation.snippet || null,
              citation_index: event.citation.index,
              created_at: new Date().toISOString(),
              sequence_order: this.sequenceOrder++
          });
          break;
    }
  }

  private async saveTurnForPersistence() {
     if (!this.conversationId) return;

     // 1. Create Assistant Message Object if not already created
     if (this.accumulatedText && !this.currentTurn.assistantMessage) {
        const message: Message = {
            id: generateUuid(),
            conversation_id: this.conversationId,
            role: "assistant",
            content: this.accumulatedText,
            created_at: new Date().toISOString(),
            sequence_order: this.sequenceOrder++,
            metadata: {}
        };
        this.currentTurn.assistantMessage = message;
     } else if (this.currentTurn.assistantMessage) {
        // Update content if it was streaming
        this.currentTurn.assistantMessage.content = this.accumulatedText;
     }
     
     // 2. Add accumulated thinking/reasoning if present
     if (this.accumulatedThinking) {
         this.currentTurn.thinkingBlocks.push({
             id: generateUuid(),
             conversation_id: this.conversationId,
             message_id: this.currentTurn.assistantMessage?.id || null,
             content: this.accumulatedThinking,
             is_complete: true,
             created_at: new Date().toISOString(),
             sequence_order: this.sequenceOrder++
         });
     }

     if (this.accumulatedReasoning) {
         this.currentTurn.reasoningBlocks.push({
             id: generateUuid(),
             conversation_id: this.conversationId,
             message_id: this.currentTurn.assistantMessage?.id || null,
             content: this.accumulatedReasoning,
             is_complete: true,
             created_at: new Date().toISOString(),
             sequence_order: this.sequenceOrder++
         });
     }

     // 3. Save the entire turn (User + Assistant + Tools + Thinking)
     // The store handles upserts, so re-saving userMessage is safe.
     await pgliteStore.saveConversationTurn(this.conversationId, this.currentTurn);
  }

  // ---------------------------------------------------------------------------
  // Public API & Conversation Management
  // ---------------------------------------------------------------------------

  async addUserMessage(content: string): Promise<void> {
    if (!content.trim()) return;

    if (!this.conversationId) {
      const conv = await pgliteStore.createConversation();
      this.conversationId = conv.id;
      window.dispatchEvent(new CustomEvent('conversation-created', { detail: { conversationId: conv.id } }));
    }

    const message: Message = {
      id: generateUuid(),
      conversation_id: this.conversationId,
      role: "user",
      content: content.trim(),
      created_at: new Date().toISOString(),
      sequence_order: this.sequenceOrder++,
      metadata: {}
    };

    // Render immediately
    this.view?.upsertItem({
        id: message.id,
        kind: "message",
        role: "user",
        html: renderMarkdown(message.content)
    });
    
    // Persist
    await pgliteStore.addMessage(message);
    this.currentTurn.userMessage = message;
  }

  async loadConversation(id: string): Promise<void> {
    try {
      const history = await pgliteStore.loadConversation(id);
      this.conversationId = id;
      this.view?.reset();
      
      // Render history
      for (const item of history.items) {
         switch (item.type) {
             case 'message':
                if (item.data.role === 'error') {
                    this.view?.upsertItem({
                        id: item.data.id || createUniqueId(),
                        kind: "error",
                        content: item.data.content
                    });
                } else {
                    this.view?.upsertItem({
                        id: item.data.id || createUniqueId(),
                        kind: "message",
                        role: item.data.role as "user" | "assistant" | "tool",
                        content: item.data.content,
                        html: renderMarkdown(item.data.content)
                    });
                }
                break;
             case 'tool_call':
                 this.view?.upsertItem({
                     id: item.data.id || `tool-${createUniqueId()}`,
                     kind: "tool_call",
                     name: item.data.tool_name,
                     args: JSON.stringify(item.data.arguments, null, 2),
                     isComplete: true
                 });
                 break;
              // citations, thinking, etc. can be added here
         }
      }
    } catch (e) {
        console.error("[chat-stream] Failed to load conversation", e);
    }
  }
  
  async createNewConversation(): Promise<void> {
       const conv = await pgliteStore.createConversation("New Conversation");
       this.conversationId = conv.id;
       this.view?.reset();
       window.dispatchEvent(new CustomEvent('conversation-updated', { detail: { conversationId: conv.id } }));
  }

  private handleConversationChanged(e: CustomEvent) {
      if (e.detail.conversationId) {
          this.loadConversation(e.detail.conversationId);
      }
  }

  private createEmptyTurn(): ConversationTurn {
    return {
      userMessage: null,
      assistantMessage: null,
      thinkingBlocks: [],
      reasoningBlocks: [],
      toolCalls: [],
      toolResults: [],
      citations: []
    };
  }
}

customElements.define("chat-stream", ChatStream);
