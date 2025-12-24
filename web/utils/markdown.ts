/**
 * Markdown rendering utilities.
 */

import { marked, type MarkedOptions } from "marked";
import { markedHighlight } from "marked-highlight";
import hljs from "highlight.js";
import { sanitizeHtml } from "./html";

// Singleton for initialization state
let isInitialized = false;

/**
 * Initialize the markdown renderer with syntax highlighting.
 * Should be called once at application startup.
 */
export function initializeMarkdown(): void {
  if (isInitialized) {
    return;
  }

  marked.use(
    markedHighlight({
      langPrefix: "hljs language-",
      highlight(code: string, lang: string): string {
        if (lang && hljs.getLanguage(lang)) {
          return hljs.highlight(code, { language: lang }).value;
        }
        return hljs.highlightAuto(code).value;
      },
    }),
  );

  isInitialized = true;
}

/**
 * Render markdown to HTML with syntax highlighting.
 * Output is sanitized for XSS protection.
 */
export function renderMarkdown(markdown: string): string {
  if (!isInitialized) {
    initializeMarkdown();
  }

  const options: MarkedOptions = {
    breaks: true,
    gfm: true,
  };

  const raw = marked.parse(markdown, options);

  // Handle both string and Promise returns from marked
  if (typeof raw === "string") {
    return sanitizeHtml(raw);
  }

  // This shouldn't happen with sync mode, but handle it gracefully
  console.warn("[markdown] Unexpected async result from marked.parse");
  return "";
}

/**
 * Render inline markdown (no block elements).
 */
export function renderInlineMarkdown(markdown: string): string {
  if (!isInitialized) {
    initializeMarkdown();
  }

  const raw = marked.parseInline(markdown);

  if (typeof raw === "string") {
    return sanitizeHtml(raw);
  }

  return "";
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
 * Extract code blocks from markdown for special handling.
 */
export interface CodeBlock {
  language: string;
  code: string;
  isMermaid: boolean;
}

export function extractCodeBlocks(markdown: string): CodeBlock[] {
  const codeBlockRegex = /```(\w*)\n([\s\S]*?)```/g;
  const blocks: CodeBlock[] = [];
  let match: RegExpExecArray | null = null;

  match = codeBlockRegex.exec(markdown);
  while (match !== null) {
    const language = match[1] || "";
    const code = match[2] || "";
    const isMermaid = language === "mermaid" || isProbablyMermaid(code);

    blocks.push({ language, code, isMermaid });
    match = codeBlockRegex.exec(markdown);
  }

  return blocks;
}

/**
 * Highlight code without wrapping in pre/code tags.
 */
export function highlightCode(code: string, language: string): string {
  if (language && hljs.getLanguage(language)) {
    return hljs.highlight(code, { language }).value;
  }
  return hljs.highlightAuto(code).value;
}
