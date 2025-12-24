/**
 * UUID Utility
 *
 * Generates UUIDs for client-side message and entity IDs.
 */

/**
 * Generate a UUID v4.
 * Uses crypto.randomUUID() if available, otherwise falls back to a custom implementation.
 */
export function generateUuid(): string {
  // Use native crypto.randomUUID() if available (modern browsers)
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }

  // Fallback implementation for older browsers
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Generate a short ID (for display purposes).
 */
export function generateShortId(): string {
  return `${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Create a unique ID with optional prefix
 */
export function createUniqueId(prefix: string = ""): string {
  const uuid = generateUuid();
  return prefix ? `${prefix}-${uuid}` : uuid;
}
