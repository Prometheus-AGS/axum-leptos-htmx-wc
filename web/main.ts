/**
 * Main entry point for the application.
 *
 * This file initializes all Web Components, third-party libraries, and PGlite database.
 */

// Third-party library initialization
import mermaid from "mermaid";
import { initializeMarkdown } from "./utils/markdown";

// Web Components
import { ChatStream } from "./components/chat-stream/chat-stream";
import { ChatMessage } from "./components/chat-message/chat-message";
import { ChatToolCall } from "./components/chat-tool-call/chat-tool-call";
import { ChatToolResult } from "./components/chat-tool-result/chat-tool-result";
import { ChatCodeBlock } from "./components/chat-code-block/chat-code-block";
import { ChatMermaid } from "./components/chat-mermaid/chat-mermaid";
import { CopyButton } from "./components/copy-button/copy-button";
import { ThemeSwitcher } from "./components/theme-switcher/theme-switcher";
import { TokenCounter } from "./components/token-counter/token-counter";
import { ConversationSidebar } from "./components/conversation-sidebar/conversation-sidebar";
import { SessionRestoreDialog } from "./components/session-restore-dialog/session-restore-dialog";
import { FileUpload } from "./components/file-upload/file-upload";

// PGlite Store
import { pgliteStore } from "./stores/pglite-store";

// Model Info Cache
import { modelInfoCache } from "./utils/model-info";

// Initialize mermaid
mermaid.initialize({
  startOnLoad: false,
  theme: "dark",
  securityLevel: "strict",
});

// Make mermaid available globally for dynamic rendering
declare global {
  interface Window {
    mermaid: typeof mermaid;
    Alpine: any;
  }
}
window.mermaid = mermaid;

// Initialize markdown renderer
initializeMarkdown();

// Initialize PGlite database
async function initializeDatabase(): Promise<void> {
  console.log("[app] Initializing PGlite database...");
  
  // Show loading indicator
  showLoadingIndicator("Initializing database...");
  
  try {
    await pgliteStore.init();
    console.log("[app] PGlite database initialized successfully");
  } catch (error) {
    console.error("[app] Failed to initialize PGlite:", error);
    showLoadingIndicator("Database initialization failed. Please refresh the page.", true);
    throw error;
  } finally {
    // Hide loading indicator after a short delay to prevent flashing
    setTimeout(() => hideLoadingIndicator(), 300);
  }
}

// Initialize model information cache
async function initializeModelInfo(): Promise<void> {
  console.log("[app] Initializing model information cache...");
  try {
    await modelInfoCache.init();
    console.log("[app] Model information cache initialized");
  } catch (error) {
    console.error("[app] Failed to initialize model info cache:", error);
    // Don't throw - allow app to continue with estimation
  }
}

// Show loading indicator
function showLoadingIndicator(message: string, isError = false): void {
  let indicator = document.getElementById("app-loading-indicator");
  
  if (!indicator) {
    indicator = document.createElement("div");
    indicator.id = "app-loading-indicator";
    indicator.className = "fixed inset-0 z-50 flex items-center justify-center bg-background";
    document.body.appendChild(indicator);
  }
  
  indicator.innerHTML = `
    <div class="flex flex-col items-center gap-4 p-8">
      ${isError ? `
        <svg class="w-12 h-12 text-danger" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
      ` : `
        <svg class="w-12 h-12 text-primary animate-spin" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
        </svg>
      `}
      <p class="text-lg font-medium ${isError ? 'text-danger' : 'text-textPrimary'}">${message}</p>
      ${!isError ? '<p class="text-sm text-textMuted">This may take a moment on first load</p>' : ''}
    </div>
  `;
}

// Hide loading indicator
function hideLoadingIndicator(): void {
  const indicator = document.getElementById("app-loading-indicator");
  if (indicator) {
    indicator.style.opacity = "0";
    indicator.style.transition = "opacity 200ms";
    setTimeout(() => indicator.remove(), 200);
  }
}

// Initialize Alpine.js global store (after PGlite is ready)
async function initializeAlpineStore(): Promise<void> {
  document.addEventListener("alpine:init", async () => {
    // Ensure PGlite is initialized
    await pgliteStore.init();
    
    // Define store type
    interface ChatStore {
      activeConversationId: string | null;
      sessionId: string | null;
      status: "idle" | "connecting" | "streaming" | "error";
      tokenUsage: {
        input: number;
        output: number;
        total: number;
        limit: number;
        isEstimate: boolean;
        cost: number;
      };
      init(this: ChatStore): Promise<void>;
      setSessionId(this: ChatStore, sessionId: string): void;
      updateTokenUsage(this: ChatStore, input: number, output: number, limit: number, isEstimate?: boolean, cost?: number): void;
      getStore(): typeof pgliteStore;
    }
    
    // Define store with proper typing
    const chatStore: ChatStore = {
      // Active conversation
      activeConversationId: null,
      sessionId: null,
      
      // Connection status
      status: "idle",
      
      // Token usage tracking
      tokenUsage: {
        input: 0,
        output: 0,
        total: 0,
        limit: 128000,
        isEstimate: true,
        cost: 0,
      },
      
      // Initialize store
      async init() {
        // Load most recent conversation
        const conversations = await pgliteStore.listConversations(1);
        if (conversations.length > 0 && conversations[0]) {
          const conversationId = conversations[0].id;
          this.activeConversationId = conversationId;
          
          // Notify chat-stream to load this conversation
          window.dispatchEvent(new CustomEvent('conversation-changed', {
            detail: { conversationId }
          }));
        }
        
        console.log("[chat-store] Initialized with PGlite");
      },
      
      // Set session ID
      setSessionId(sessionId: string): void {
        this.sessionId = sessionId;
        console.log("[chat-store] Session ID set:", sessionId);
      },
      
      // Update token usage
      updateTokenUsage(input: number, output: number, limit: number, isEstimate: boolean = true, cost: number = 0): void {
        this.tokenUsage = {
          input,
          output,
          total: input + output,
          limit,
          isEstimate,
          cost,
        };
        console.log("[chat-store] Token usage updated:", this.tokenUsage);
      },
      
      // Get the PGlite store instance
      getStore() {
        return pgliteStore;
      },
    };
    
    window.Alpine.store("chat", chatStore);
  });
}

// Register Web Components
function registerComponents(): void {
  // Check if already registered to avoid errors on hot reload
  const components = [
    { name: "chat-stream", component: ChatStream },
    { name: "chat-message", component: ChatMessage },
    { name: "chat-tool-call", component: ChatToolCall },
    { name: "chat-tool-result", component: ChatToolResult },
    { name: "chat-code-block", component: ChatCodeBlock },
    { name: "chat-mermaid", component: ChatMermaid },
    { name: "copy-button", component: CopyButton },
    { name: "theme-switcher", component: ThemeSwitcher },
    { name: "token-counter", component: TokenCounter },
    { name: "conversation-sidebar", component: ConversationSidebar },
    { name: "session-restore-dialog", component: SessionRestoreDialog },
    { name: "file-upload", component: FileUpload },
  ];
  
  for (const { name, component } of components) {
    if (!customElements.get(name)) {
      customElements.define(name, component);
    }
  }
}

// Initialize everything
async function initialize(): Promise<void> {
  // Initialize database FIRST before anything else
  await initializeDatabase();
  
  // Initialize model info cache (in parallel, non-blocking)
  await initializeModelInfo();
  
  // Register components after database is ready
  registerComponents();
  
  // Initialize Alpine store
  await initializeAlpineStore();
  
  // Start Alpine.js (check if already initialized)
  try {
    if (typeof window.Alpine.store === 'function') {
      // Alpine is already initialized
      console.log("[app] Alpine.js already initialized");
    } else {
      // Start Alpine
      window.Alpine.start();
      console.log("[app] Alpine.js started");
    }
  } catch (e) {
    // Alpine not loaded yet or error
    console.warn("[app] Alpine.js initialization check failed:", e);
  }
  
  console.log("[app] Application initialized");
}

// Start initialization
initialize().catch((error) => {
  console.error("[app] Initialization failed:", error);
});
