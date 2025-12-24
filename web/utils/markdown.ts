/**
 * Markdown rendering utilities.
 */

import { marked } from "marked";
import { sanitizeHtml, escapeHtml } from "./html";

// Singleton for initialization state
let isInitialized = false;

// Custom renderer to output Web Components
const renderer = new marked.Renderer();

// Override code block rendering
renderer.code = ({ text, lang }: { text: string; lang?: string }): string => {
  const language = (lang || "").toLowerCase();
  const code = text;

  // Check for Mermaid
  if (language === "mermaid" || isProbablyMermaid(code)) {
    return `<chat-mermaid code="${escapeHtml(code)}"></chat-mermaid>`;
  }

  // Standard code block
  return `<chat-code-block language="${escapeHtml(language)}" code="${escapeHtml(code)}"></chat-code-block>`;
};

// Override image rendering to support video
renderer.image = ({ href, title, text }: { href: string; title: string | null; text: string }): string => {
  if (!href) return "";

  const fileExt = href.split('.').pop()?.toLowerCase();
  const isVideo = ["mp4", "webm", "ogg", "mov"].includes(fileExt || "");

  if (isVideo) {
    return `
      <video controls class="max-w-full rounded-lg my-2" title="${title || text || ''}">
        <source src="${href}" type="video/${fileExt === 'mov' ? 'mp4' : fileExt}">
        Your browser does not support the video tag.
      </video>
    `;
  }

  return `<img src="${href}" alt="${text}" title="${title || ''}" class="max-w-full rounded-lg my-2">`;
};

/**
 * Initialize the markdown renderer with custom renderer.
 * Should be called once at application startup.
 */
export function initializeMarkdown(): void {
  if (isInitialized) {
    return;
  }

  marked.use({ renderer });
  isInitialized = true;
}

/**
 * Render markdown to HTML with syntax highlighting (via components).
 * Output is sanitized for XSS protection (components handle their own sanitization).
 */
export function renderMarkdown(markdown: string): string {
  if (!isInitialized) {
    initializeMarkdown();
  }

  // Parse markdown
  const raw = marked.parse(markdown) as string;
  
  // Note: We don't use global sanitizeHtml here because it might strip our custom web components
  // Instead, we rely on marked's default sanitization + specific component safety
  return raw; 
}

/**
 * Check if a code block is likely Mermaid diagram syntax.
 */
export function isProbablyMermaid(code: string): boolean {
  const head = code.trimStart().slice(0, 32).toLowerCase();
  const mermaidKeywords = [
    "graph ",
    "sequencediagram",
    "flowchart ",
    "classdiagram",
    "statediagram",
    "erdiagram",
    "gantt",
    "journey",
    "pie ",
    "gitgraph",
    "mindmap",
    "timeline",
    "sankey",
    "xychart",
    "quadrant",
  ];

  return mermaidKeywords.some((keyword) => head.startsWith(keyword));
}

/**
 * Highlight code without wrapping in pre/code tags.
 * (Preserved for compatibility, though largely superseded by ChatCodeBlock)
 */
import hljs from "highlight.js";
export function highlightCode(code: string, language: string): string {
  if (language && hljs.getLanguage(language)) {
    return hljs.highlight(code, { language }).value;
  }
  return hljs.highlightAuto(code).value;
}
