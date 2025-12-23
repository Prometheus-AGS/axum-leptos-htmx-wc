/**
 * HTML utility functions.
 */

import DOMPurify from "dompurify";

/**
 * Escape HTML special characters to prevent XSS.
 */
export function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/**
 * Sanitize HTML content using DOMPurify.
 */
export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: [
      "p",
      "br",
      "strong",
      "em",
      "code",
      "pre",
      "blockquote",
      "ul",
      "ol",
      "li",
      "a",
      "h1",
      "h2",
      "h3",
      "h4",
      "h5",
      "h6",
      "span",
      "div",
      "table",
      "thead",
      "tbody",
      "tr",
      "th",
      "td",
    ],
    ALLOWED_ATTR: ["href", "class", "id", "target", "rel"],
    ALLOW_DATA_ATTR: false,
  });
}

/**
 * Create a unique ID for DOM elements.
 */
export function createUniqueId(prefix: string = "id"): string {
  return `${prefix}-${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Parse JSON safely with type inference.
 */
export function safeJsonParse<T>(json: string): T | null {
  try {
    return JSON.parse(json) as T;
  } catch {
    return null;
  }
}

/**
 * Format JSON for display with syntax highlighting classes.
 */
export function formatJsonForDisplay(json: string): string {
  try {
    const parsed: unknown = JSON.parse(json);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return json;
  }
}
