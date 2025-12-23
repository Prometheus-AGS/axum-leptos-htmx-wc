/**
 * Clipboard utility functions.
 */

/**
 * Copy text to the clipboard.
 * Returns true on success, false on failure.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  // Try the modern Clipboard API first
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (err) {
      console.warn("[clipboard] Clipboard API failed:", err);
    }
  }

  // Fallback to execCommand for older browsers
  return copyToClipboardFallback(text);
}

/**
 * Fallback clipboard copy using deprecated execCommand.
 */
function copyToClipboardFallback(text: string): boolean {
  const textarea = document.createElement("textarea");
  textarea.value = text;

  // Make the textarea invisible
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  textarea.style.top = "-9999px";
  textarea.setAttribute("readonly", "");

  document.body.appendChild(textarea);

  try {
    textarea.select();
    const success = document.execCommand("copy");
    return success;
  } catch (err) {
    console.warn("[clipboard] execCommand fallback failed:", err);
    return false;
  } finally {
    document.body.removeChild(textarea);
  }
}

/**
 * Read text from the clipboard.
 * Returns null if reading fails.
 */
export async function readFromClipboard(): Promise<string | null> {
  if (navigator.clipboard && window.isSecureContext) {
    try {
      return await navigator.clipboard.readText();
    } catch (err) {
      console.warn("[clipboard] Failed to read clipboard:", err);
      return null;
    }
  }

  // No fallback for reading
  return null;
}
