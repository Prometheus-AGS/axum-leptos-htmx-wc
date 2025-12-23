/**
 * Main entry point for the application.
 *
 * This file initializes all Web Components and third-party libraries.
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
  }
}
window.mermaid = mermaid;

// Initialize markdown renderer
initializeMarkdown();

// Register Web Components
function registerComponents(): void {
  // Core chat components
  customElements.define("chat-stream", ChatStream);
  customElements.define("chat-message", ChatMessage);
  customElements.define("chat-tool-call", ChatToolCall);
  customElements.define("chat-tool-result", ChatToolResult);
  customElements.define("chat-code-block", ChatCodeBlock);
  customElements.define("chat-mermaid", ChatMermaid);
  customElements.define("copy-button", CopyButton);
}

// Register all components
registerComponents();

// Log initialization
console.log("[app] Web Components initialized");
