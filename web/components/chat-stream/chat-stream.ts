/**
 * Chat Stream Web Component (Enhanced with PGlite & Streaming Optimizer)
 *
 * Manages SSE connection, renders chat messages with smooth debouncing,
 * and persists all events to PGlite database.
 */

import type { NormalizedEvent } from "../../types/events";
import type { ChatState, ChatItem, ToolCallItem } from "../../types/chat";
import { createInitialChatState } from "../../types/chat";
import { SSEConnection } from "../../utils/sse";
import { renderMarkdown } from "../../utils/markdown";
import { escapeHtml, formatJsonForDisplay, createUniqueId } from "../../utils/html";
import { generateUuid } from "../../utils/uuid";
import { pgliteStore } from "../../stores/pglite-store";
import type { ConversationTurn, Message, ThinkingBlock, ReasoningBlock, ToolCall, ToolResult, Citation } from "../../types/database";
import { StreamingOptimizer, StreamVelocityTracker, IncrementalMarkdownParser } from "../../utils/streaming-optimizer";

/**
 * Chat Stream component with PGlite persistence and smooth streaming.
 */
export class ChatStream extends HTMLElement {
  private state: ChatState = createInitialChatState();
  private connection: SSEConnection | null = null;
  private transcriptEl: HTMLElement | null = null;
  private statusEl: HTMLElement | null = null;
  
  // PGlite and conversation management
  private conversationId: string | null = null;
  private currentTurn: ConversationTurn = this.createEmptyTurn();
  private sequenceOrder = 0;
  
  // Streaming optimization
  private streamingOptimizer = new StreamingOptimizer();
  private velocityTracker = new StreamVelocityTracker();
  private markdownParser = new IncrementalMarkdownParser();
  private stableMarkdownCache = new Map<string, string>();

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
    this.render();
    this.setupElements();
    
    // Listen for conversation changes from sidebar
    window.addEventListener('conversation-changed', this.handleConversationChanged as EventListener);
  }

  disconnectedCallback(): void {
    this.disconnect();
    this.streamingOptimizer.cancel();
    window.removeEventListener('conversation-changed', this.handleConversationChanged as EventListener);
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue !== newValue) {
      if (name === "stream-url" || name === "session-id") {
        this.disconnect();
      }
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

  /**
   * Add a user message to the conversation.
   * This is called immediately when the user submits a message.
   */
  async addUserMessage(content: string): Promise<void> {
    if (!content.trim()) return;

    // Create or get conversation
    if (!this.conversationId) {
      const conv = await pgliteStore.createConversation();
      this.conversationId = conv.id;
      
      // Notify sidebar to refresh
      window.dispatchEvent(new CustomEvent('conversation-created', { detail: { conversationId: conv.id } }));
    }

    // Create message
    const message: Message = {
      id: generateUuid(),
      conversation_id: this.conversationId,
      role: "user",
      content: content.trim(),
      created_at: new Date().toISOString(),
      sequence_order: this.sequenceOrder++,
      metadata: {}
    };

    // Store in current turn
    this.currentTurn.userMessage = message;

    // Add to local state for immediate display
    this.state.items.push({
      kind: "message",
      role: "user",
      content: message.content,
      html: renderMarkdown(message.content),
    });

    this.renderTranscript();
    console.log("[chat-stream] Added user message:", message.id);
  }

  /**
   * Create a new conversation and clear the UI.
   */
  async createNewConversation(): Promise<void> {
    try {
      // Create new conversation in PGlite
      const newConversation = await pgliteStore.createConversation("New Conversation");
      this.conversationId = newConversation.id;
      
      // Clear UI state
      this.state.items = [];
      this.state.streamingText = "";
      this.state.streamingThinking = "";
      this.state.streamingReasoning = "";
      this.state.toolCalls.clear();
      this.state.citations = [];
      this.state.status = "idle";
      
      // Reset current turn
      this.currentTurn = this.createEmptyTurn();
      
      // Render empty transcript
      this.renderTranscript();
      
      // Notify sidebar to refresh
      window.dispatchEvent(new CustomEvent('conversation-updated', { 
        detail: { conversationId: this.conversationId } 
      }));
      
      // Dispatch conversation-changed event
      window.dispatchEvent(new CustomEvent('conversation-changed', {
        detail: { conversationId: this.conversationId }
      }));
      
      console.log("[chat-stream] Created new conversation:", this.conversationId);
    } catch (error) {
      console.error("[chat-stream] Failed to create new conversation:", error);
    }
  }

  /**
   * Load a conversation from PGlite and render its history.
   */
  async loadConversation(id: string): Promise<void> {
    try {
      const history = await pgliteStore.loadConversation(id);
      this.conversationId = id;
      this.state.items = [];

      // Convert history items to ChatItems
      for (const item of history.items) {
        switch (item.type) {
          case 'message':
            this.state.items.push({
              kind: "message",
              role: item.data.role,
              content: item.data.content,
              html: renderMarkdown(item.data.content),
            });
            break;
          
          case 'thinking':
            this.state.items.push({
              kind: "thinking",
              content: item.data.content,
              isComplete: item.data.is_complete,
            });
            break;
          
          case 'reasoning':
            this.state.items.push({
              kind: "reasoning",
              content: item.data.content,
              isComplete: item.data.is_complete,
            });
            break;
          
          case 'tool_call':
            this.state.items.push({
              kind: "tool_call",
              id: item.data.id,
              name: item.data.tool_name,
              arguments: item.data.arguments,
              argumentsJson: JSON.stringify(item.data.arguments, null, 2),
            });
            break;
          
          case 'tool_result':
            this.state.items.push({
              kind: "tool_result",
              toolCallId: item.data.tool_call_id || "",
              name: item.data.tool_name || "unknown",
              result: item.data.content || "",
              success: item.data.success ?? false,
            });
            break;
          
          case 'citation':
            // Citations are typically grouped, but we can add them individually
            this.state.citations.push({
              url: item.data.url,
              title: item.data.title || undefined,
              index: item.data.citation_index,
            });
            break;
        }
      }

      this.renderTranscript();
      console.log("[chat-stream] Loaded conversation:", id, "with", history.items.length, "items");
    } catch (error) {
      console.error("[chat-stream] Failed to load conversation:", error);
    }
  }

  /**
   * Start the SSE stream.
   */
  startStream(responseJson?: string): void {
    if (responseJson) {
      try {
        const response = JSON.parse(responseJson) as {
          session_id?: string;
          stream_url?: string;
        };
        if (response.stream_url) {
          this.setAttribute("stream-url", response.stream_url);
        }
        
        // Update session ID in conversation
        if (response.session_id && this.conversationId) {
          pgliteStore.getConversation(this.conversationId).then(conv => {
            if (conv) {
              // Update server_session_id in database
              // (We'll need to add this method to pgliteStore)
            }
          });
        }
      } catch {
        console.warn("[chat-stream] Failed to parse response JSON");
      }
    }

    this.connect();
  }

  /**
   * Connect to the SSE stream.
   */
  connect(): void {
    this.disconnect();
    
    // Reset streaming state (but keep messages)
    this.state.streamingText = "";
    this.state.streamingThinking = "";
    this.state.streamingReasoning = "";
    this.state.toolCalls.clear();
    this.state.citations = [];
    this.state.status = "connecting";
    this.updateStatus("Connecting...");
    
    // Reset streaming optimizers
    this.velocityTracker.reset();
    this.markdownParser.reset();
    this.stableMarkdownCache.clear();
    
    this.renderTranscript();

    this.connection = new SSEConnection({
      url: this.streamUrl,
      handlers: {
        onOpen: () => {
          this.state.status = "streaming";
          console.log("[chat-stream] SSE connection opened");
        },
        onNormalizedEvent: (event) => this.handleEvent(event),
        onError: (error) => {
          this.state.status = "error";
          console.error("[chat-stream] SSE connection error:", error);
          this.state.items.push({
            kind: "error",
            message: "Connection error. Please try again.",
          });
          this.renderTranscript();
        },
        onClose: () => {
          console.log("[chat-stream] SSE connection closed, status:", this.state.status);
          // Don't show any status message on close - token counter shows completion
        },
      },
    });

    this.connection.connect();
  }

  /**
   * Disconnect from the SSE stream.
   */
  disconnect(): void {
    if (this.connection) {
      this.connection.disconnect();
      this.connection = null;
    }
  }

  /**
   * Handle a normalized event from the SSE stream.
   */
  private handleEvent(event: NormalizedEvent): void {
    switch (event.type) {
      case "stream.start":
        this.state.requestId = event.data.request_id;
        break;

      case "message.delta":
        this.handleMessageDelta(event.data.text);
        break;

      case "thinking.delta":
        this.handleThinkingDelta(event.data.text);
        break;

      case "reasoning.delta":
        this.handleReasoningDelta(event.data.text);
        break;

      case "citation.added":
        this.state.citations.push(event.data);
        break;

      case "memory.update":
        console.debug("[chat-stream] Memory update:", event.data);
        break;

      case "tool_call.delta":
        this.handleToolCallDelta(event.data);
        break;

      case "tool_call.complete":
        this.handleToolCallComplete(event.data);
        break;

      case "tool_result":
        this.handleToolResult(event.data);
        break;

      case "error":
        this.handleError(event.data);
        break;

      case "usage":
        this.handleUsage(event.data);
        break;

      case "done":
        this.handleDone();
        break;
    }
  }

  /**
   * Handle message delta with RAF batching and incremental parsing.
   */
  private handleMessageDelta(text: string): void {
    const streamId = 'assistant-message';
    
    // Track velocity for adaptive batching
    this.velocityTracker.recordChunk(text.length);
    
    // Buffer chunk and schedule RAF flush
    this.streamingOptimizer.bufferTextChunk(
      streamId,
      text,
      (flushedText) => {
        this.state.streamingText += flushedText;
        
        // Use incremental markdown parser
        const html = this.markdownParser.parse(this.state.streamingText, renderMarkdown);
        
        // Update or create assistant message item
        const lastItem = this.state.items[this.state.items.length - 1];
        if (lastItem?.kind === "message" && lastItem.role === "assistant") {
          lastItem.content = this.state.streamingText;
          lastItem.html = html;
        } else {
          this.state.items.push({
            kind: "message",
            role: "assistant",
            content: this.state.streamingText,
            html: html,
          });
        }
        
        this.renderTranscript();
      }
    );
  }

  /**
   * Handle thinking delta with larger batches.
   */
  private handleThinkingDelta(text: string): void {
    const streamId = 'thinking-block';
    
    this.streamingOptimizer.bufferTextChunk(
      streamId,
      text,
      (flushedText) => {
        this.state.streamingThinking += flushedText;
        
        // Find or create thinking item
        let thinkingItem = this.state.items.find(
          (item) => item.kind === "thinking" && !item.isComplete
        );
        
        if (!thinkingItem) {
          thinkingItem = {
            kind: "thinking",
            content: this.state.streamingThinking,
            isComplete: false,
          };
          this.state.items.push(thinkingItem);
        } else {
          thinkingItem.content = this.state.streamingThinking;
        }
        
        this.renderTranscript();
      }
    );
  }

  /**
   * Handle reasoning delta with larger batches.
   */
  private handleReasoningDelta(text: string): void {
    const streamId = 'reasoning-block';
    
    this.streamingOptimizer.bufferTextChunk(
      streamId,
      text,
      (flushedText) => {
        this.state.streamingReasoning += flushedText;
        
        // Find or create reasoning item
        let reasoningItem = this.state.items.find(
          (item) => item.kind === "reasoning" && !item.isComplete
        );
        
        if (!reasoningItem) {
          reasoningItem = {
            kind: "reasoning",
            content: this.state.streamingReasoning,
            isComplete: false,
          };
          this.state.items.push(reasoningItem);
        } else {
          reasoningItem.content = this.state.streamingReasoning;
        }
        
        this.renderTranscript();
      }
    );
  }

  /**
   * Handle tool call delta (debounced for structured data).
   */
  private handleToolCallDelta(data: {
    index: number;
    id?: string;
    name?: string;
    arguments?: string;
  }): void {
    const acc = this.state.toolCalls.get(data.index) || {
      index: data.index,
      id: data.id || "",
      name: data.name || "",
      argumentsJson: "",
    };

    if (data.id) acc.id = data.id;
    if (data.name) acc.name = data.name;
    if (data.arguments) acc.argumentsJson += data.arguments;

    this.state.toolCalls.set(data.index, acc);
    
    // Find or create tool call item in state.items for real-time display
    let toolCallItem = this.state.items.find(
      (item) => item.kind === "tool_call" && item.id === acc.id
    ) as ToolCallItem | undefined;
    
    if (!toolCallItem && acc.id) {
      // Create new tool call item
      toolCallItem = {
        kind: "tool_call",
        role: "tool",
        callIndex: data.index,
        id: acc.id,
        name: acc.name,
        argumentsRaw: acc.argumentsJson,
        status: "streaming",
      };
      this.state.items.push(toolCallItem);
    } else if (toolCallItem) {
      // Update existing item
      toolCallItem.argumentsRaw = acc.argumentsJson;
      if (acc.name) toolCallItem.name = acc.name;
    }
    
    // Debounce UI update for structured data
    setTimeout(() => this.renderTranscript(), this.streamingOptimizer.config.structuredDelay);
  }

  /**
   * Handle tool call complete.
   */
  private handleToolCallComplete(data: {
    index: number;
    id: string;
    name: string;
    arguments_json: string;
  }): void {
    const acc = this.state.toolCalls.get(data.index);
    if (acc) {
      acc.id = data.id;
      acc.name = data.name;
      acc.argumentsJson = data.arguments_json;
    }

    // Find existing tool call item (created during delta) and mark as complete
    let toolCallItem = this.state.items.find(
      (item) => item.kind === "tool_call" && item.id === data.id
    ) as ToolCallItem | undefined;
    
    if (toolCallItem) {
      // Update existing item to complete
      toolCallItem.status = "complete";
      toolCallItem.argumentsRaw = data.arguments_json;
      toolCallItem.name = data.name;
    } else {
      // Create new item if it doesn't exist (shouldn't happen normally)
      toolCallItem = {
        kind: "tool_call",
        role: "tool",
        callIndex: data.index,
        id: data.id,
        name: data.name,
        argumentsRaw: data.arguments_json,
        status: "complete",
      };
      this.state.items.push(toolCallItem);
    }

    // Store in current turn for PGlite (with validation)
    if (this.conversationId && data.id && data.name && typeof data.index === 'number') {
      this.currentTurn.toolCalls.push({
        id: data.id,
        conversation_id: this.conversationId,
        message_id: "", // Will be set when assistant message is saved
        call_index: data.index,
        tool_name: data.name,
        arguments: JSON.parse(data.arguments_json || "{}"),
        status: "complete",
        created_at: new Date().toISOString(),
        sequence_order: this.sequenceOrder++
      });
    } else {
      console.warn("[chat-stream] Skipping tool call save - missing required fields:", {
        conversationId: this.conversationId,
        id: data.id,
        name: data.name,
        index: data.index
      });
    }

    this.renderTranscript();
  }

  /**
   * Handle tool result.
   */
  private handleToolResult(data: {
    tool_call_id: string;
    name: string;
    result: string;
    success: boolean;
  }): void {
    // Validate tool_call_id before proceeding
    if (!data.tool_call_id) {
      console.warn("[chat-stream] Cannot save tool result: tool_call_id is missing", data);
      return;
    }
    
    if (!this.conversationId) {
      console.warn("[chat-stream] Cannot save tool result: conversationId is null");
      return;
    }

    // Find the matching tool call and add the result to it
    const toolCallItem = this.state.items.find(
      (item) => item.kind === "tool_call" && item.id === data.tool_call_id
    ) as ToolCallItem | undefined;
    
    if (toolCallItem) {
      // Add result to existing tool call
      toolCallItem.result = {
        content: data.result,
        success: data.success,
      };
      toolCallItem.status = "complete"; // Ensure status is complete when result arrives
    } else {
      // If no matching tool call found, log warning but still save to PGlite
      console.warn("[chat-stream] No matching tool call found for result:", data.tool_call_id);
    }

    // Store in current turn for PGlite
    this.currentTurn.toolResults.push({
      id: generateUuid(),
      conversation_id: this.conversationId,
      tool_call_id: data.tool_call_id,
      tool_name: data.name,
      content: data.result,
      success: data.success,
      created_at: new Date().toISOString(),
      sequence_order: this.sequenceOrder++
    });

    this.renderTranscript();
  }

  /**
   * Handle error.
   */
  private handleError(data: { message: string }): void {
    this.state.items.push({
      kind: "error",
      message: data.message,
    });
    this.renderTranscript();
  }

  /**
   * Handle usage information from the API.
   */
  private handleUsage(data: { prompt_tokens: number; completion_tokens: number; total_tokens: number }): void {
    console.log("[chat-stream] Received usage:", data);
    
    // Update Alpine store with token usage
    const Alpine = (window as any).Alpine;
    if (Alpine) {
      const chatStore = Alpine.store('chat');
      if (chatStore && chatStore.updateTokenUsage) {
        // Get model info for context limit and cost
        const modelId = "gpt-4o"; // TODO: Get from config
        const contextLimit = 128000; // TODO: Get from model info
        const cost = 0; // TODO: Calculate from model info
        
        chatStore.updateTokenUsage(
          data.prompt_tokens,
          data.completion_tokens,
          contextLimit,
          false, // Not an estimate
          cost
        );
      }
    }
  }

  /**
   * Handle done event - save complete turn to PGlite.
   */
  private async handleDone(): Promise<void> {
    this.state.status = "done";

    // Mark thinking/reasoning as complete
    for (const item of this.state.items) {
      if (item.kind === "thinking" || item.kind === "reasoning") {
        item.isComplete = true;
      }
    }

    // Add citations if any
    if (this.state.citations.length > 0) {
      this.state.items.push({
        kind: "citations",
        citations: [...this.state.citations],
      });
    }

    // Create assistant message for PGlite
    if (this.state.streamingText && this.conversationId) {
      const assistantMessage: Message = {
        id: generateUuid(),
        conversation_id: this.conversationId,
        role: "assistant",
        content: this.state.streamingText,
        created_at: new Date().toISOString(),
        sequence_order: this.sequenceOrder++,
        metadata: {}
      };
      
      this.currentTurn.assistantMessage = assistantMessage;
      
      // Update message_id for tool calls
      for (const toolCall of this.currentTurn.toolCalls) {
        toolCall.message_id = assistantMessage.id;
      }
      
      // Save complete turn with all chunks to PGlite
      try {
        await pgliteStore.saveConversationTurn(this.conversationId, this.currentTurn);
        
        console.log("[chat-stream] Saved complete turn with all chunks:", {
          thinking: this.currentTurn.thinkingBlocks.length,
          reasoning: this.currentTurn.reasoningBlocks.length,
          toolCalls: this.currentTurn.toolCalls.length,
          toolResults: this.currentTurn.toolResults.length,
          citations: this.currentTurn.citations.length
        });
        
        // Generate title if this is the first turn
        const conv = await pgliteStore.getConversation(this.conversationId);
        if (conv && conv.title === 'New Conversation' && this.currentTurn.userMessage) {
          this.generateTitle(this.conversationId, this.currentTurn.userMessage.content);
        }
        
        // Notify sidebar to refresh
        window.dispatchEvent(new CustomEvent('conversation-updated', { 
          detail: { conversationId: this.conversationId } 
        }));
      } catch (error) {
        console.error("[chat-stream] Failed to save turn:", error);
      }
      
      // Reset for next turn
      this.currentTurn = this.createEmptyTurn();
    }

    // Don't show "Done" - token counter will show the info
    this.disconnect();
    this.renderTranscript();
  }

  /**
   * Generate title for conversation using non-streaming LLM call.
   */
  private async generateTitle(conversationId: string, firstMessage: string): Promise<void> {
    try {
      const response = await fetch('/api/generate-title', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: firstMessage })
      });
      
      if (response.ok) {
        const data = await response.json() as { title: string };
        await pgliteStore.updateTitle(conversationId, data.title);
        
        // Notify sidebar to refresh
        window.dispatchEvent(new CustomEvent('conversation-updated', { 
          detail: { conversationId } 
        }));
      }
    } catch (error) {
      console.error("[chat-stream] Failed to generate title:", error);
    }
  }

  /**
   * Handle conversation changed event from sidebar.
   */
  private handleConversationChanged = async (e: CustomEvent): Promise<void> => {
    const { conversationId } = e.detail;
    if (conversationId) {
      await this.loadConversation(conversationId);
    } else {
      // New conversation
      this.conversationId = null;
      this.state.items = [];
      this.renderTranscript();
    }
  };

  /**
   * Setup element references.
   */
  private setupElements(): void {
    this.transcriptEl = this.querySelector(".chat-stream-transcript");
    this.statusEl = this.querySelector(".chat-stream-status");
  }

  /**
   * Update status text.
   */
  private updateStatus(status: string): void {
    if (this.statusEl) {
      this.statusEl.textContent = status;
    }
  }

  /**
   * Render the component.
   */
  private render(): void {
    this.innerHTML = `
      <div class="chat-stream">
        <div class="chat-stream-transcript"></div>
        <div class="chat-stream-status"></div>
      </div>
    `;
  }

  /**
   * Render the transcript with smooth scrolling.
   */
  private renderTranscript(): void {
    if (!this.transcriptEl) return;

    const html = this.state.items.map((item) => this.renderItem(item)).join("");
    this.transcriptEl.innerHTML = html;

    // Smooth scroll to bottom
    this.smoothScrollToBottom();
  }

  /**
   * Smooth scroll with RAF and distance detection.
   */
  private smoothScrollToBottom(): void {
    if (!this.transcriptEl) return;

    const targetScroll = this.transcriptEl.scrollHeight;
    const currentScroll = this.transcriptEl.scrollTop;
    const distance = targetScroll - currentScroll;

    // Only smooth scroll if distance is small (user is following along)
    if (distance < 100) {
      this.transcriptEl.scrollTo({
        top: targetScroll,
        behavior: 'smooth'
      });
    } else if (distance > 0) {
      // Jump scroll if user has scrolled up significantly
      this.transcriptEl.scrollTop = targetScroll;
    }
  }

  /**
   * Render a single chat item.
   */
  private renderItem(item: ChatItem): string {
    switch (item.kind) {
      case "message":
        return this.renderMessage(item);
      case "thinking":
        return this.renderThinking(item);
      case "reasoning":
        return this.renderReasoning(item);
      case "tool_call":
        return this.renderToolCall(item);
      case "tool_result":
        return this.renderToolResult(item);
      case "citations":
        return this.renderCitations(item);
      case "error":
        return this.renderError(item);
      default:
        return "";
    }
  }

  private renderMessage(item: { role: string; html: string; content: string }): string {
    const roleClass = item.role === "user" ? "user" : "assistant";
    const bubbleClass = item.role === "user" ? "bg-bubbleUser" : "bg-bubbleAssistant";
    
    return `
      <div class="chat-item chat-message ${roleClass}">
        <div class="message-bubble ${bubbleClass} rounded-2xl px-4 py-3 relative">
          <button class="copy-btn absolute top-2 right-2 opacity-0 hover:opacity-100 transition-opacity" 
                  data-copy="${escapeHtml(item.content)}"
                  aria-label="Copy message">
            <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
          </button>
          <div class="prose prose-sm max-w-none">${item.html}</div>
        </div>
      </div>
    `;
  }

  private renderThinking(item: { content: string; isComplete: boolean }): string {
    const thinkingId = createUniqueId("thinking");
    return `
      <div class="chat-item chat-thinking">
        <div class="thinking-block bg-surfaceContainerHigh rounded-xl px-4 py-3 relative group">
          <div class="thinking-header flex items-center gap-2 mb-2">
            <svg class="w-4 h-4 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4"/>
              <path d="M12 8h.01"/>
            </svg>
            <span class="text-sm font-medium text-textSecondary">Thinking</span>
          </div>
          <div id="${thinkingId}" class="prose prose-sm max-w-none text-textPrimary" data-raw-content="${escapeHtml(item.content)}">${escapeHtml(item.content)}</div>
          <copy-button target="${thinkingId}" class="absolute top-2 right-2 opacity-70 group-hover:opacity-100 transition-opacity"></copy-button>
        </div>
      </div>
    `;
  }

  private renderReasoning(item: { content: string; isComplete: boolean }): string {
    const reasoningId = createUniqueId("reasoning");
    return `
      <div class="chat-item chat-reasoning">
        <div class="reasoning-block bg-surfaceContainerHigh rounded-xl px-4 py-3 relative group">
          <div class="reasoning-header flex items-center gap-2 mb-2">
            <svg class="w-4 h-4 text-info" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 1 1 7.072 0l-.548.547A3.374 3.374 0 0 0 14 18.469V19a2 2 0 1 1-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"/>
            </svg>
            <span class="text-sm font-medium text-textSecondary">Reasoning</span>
          </div>
          <div id="${reasoningId}" class="prose prose-sm max-w-none text-textPrimary" data-raw-content="${escapeHtml(item.content)}">${escapeHtml(item.content)}</div>
          <copy-button target="${reasoningId}" class="absolute top-2 right-2 opacity-70 group-hover:opacity-100 transition-opacity"></copy-button>
        </div>
      </div>
    `;
  }

  private renderToolCall(item: ToolCallItem): string {
    // Combined tool call and result in single block, centered
    const hasResult = !!item.result;
    
    return `
      <div class="flex justify-center my-4">
        <chat-tool-call 
          call-id="${item.id}"
          name="${escapeHtml(item.name)}"
          arguments='${escapeHtml(item.argumentsRaw)}'
          status="${item.status}"
          result='${hasResult ? escapeHtml(JSON.stringify(item.result)) : ''}'
          has-result="${hasResult}">
        </chat-tool-call>
      </div>
    `;
  }

  private renderToolResult(item: { toolCallId: string; name: string; result: string; success: boolean }): string {
    // This should not be called anymore since results are combined with calls
    // But keep for backwards compatibility with loaded conversations
    return `
      <chat-tool-result
        tool-call-id="${escapeHtml(item.toolCallId)}"
        tool-name="${escapeHtml(item.name)}"
        success="${item.success ?? false}"
        result='${escapeHtml(item.result)}'>
      </chat-tool-result>
    `;
  }

  private renderCitations(item: { citations: Array<{ url: string; title?: string; index: number }> }): string {
    const citationsHtml = item.citations
      .map((c) => `
        <li class="citation-item">
          <a href="${escapeHtml(c.url)}" target="_blank" rel="noopener noreferrer" class="citation-link">
            <span class="citation-index">[${c.index}]</span>
            <span class="citation-title">${escapeHtml(c.title || c.url)}</span>
          </a>
        </li>
      `)
      .join("");

    return `
      <div class="chat-item chat-citations">
        <div class="citations-block bg-surfaceContainerHigh rounded-xl px-4 py-3">
          <div class="citations-header flex items-center gap-2 mb-2">
            <svg class="w-4 h-4 text-info" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="16" y1="13" x2="8" y2="13"/>
              <line x1="16" y1="17" x2="8" y2="17"/>
              <polyline points="10 9 9 9 8 9"/>
            </svg>
            <span class="text-sm font-medium text-textSecondary">Citations</span>
          </div>
          <ul class="citations-list space-y-1">${citationsHtml}</ul>
        </div>
      </div>
    `;
  }

  private renderError(item: { message: string }): string {
    return `
      <div class="chat-item chat-error">
        <div class="error-block bg-danger/10 border border-danger/20 rounded-xl px-4 py-3">
          <div class="error-header flex items-center gap-2 mb-2">
            <svg class="w-4 h-4 text-danger" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            <span class="text-sm font-medium text-danger">Error</span>
          </div>
          <p class="text-sm text-danger">${escapeHtml(item.message)}</p>
        </div>
      </div>
    `;
  }
}

// Register the custom element
customElements.define("chat-stream", ChatStream);

