import { describe, expect, test } from "bun:test";

import { createUniqueId, generateShortId, generateUuid } from "../utils/uuid";
import {
  calculateContextPercentage,
  estimateConversationTokens,
  estimateTokens,
  formatCost,
  formatTokenCount,
  getUsageBackgroundClass,
  getUsageColorClass,
} from "../utils/token-counter";

describe("uuid utils", () => {
  test("generateUuid returns a v4 uuid", () => {
    const uuid = generateUuid();
    expect(uuid).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    );
  });

  test("createUniqueId prefixes when provided", () => {
    const id = createUniqueId("message");
    expect(id.startsWith("message-")).toBe(true);
  });

  test("generateShortId includes a timestamp separator", () => {
    const id = generateShortId();
    expect(id.includes("_")).toBe(true);
  });
});

describe("token counter utils", () => {
  test("estimateTokens uses model heuristics", () => {
    expect(estimateTokens("abcd", "gpt-4")).toBe(1);
    expect(estimateTokens("abcdefg", "claude-3")).toBe(2);
    expect(estimateTokens("abcdefgh")).toBe(2);
  });

  test("estimateConversationTokens accounts for overhead", () => {
    const total = estimateConversationTokens(
      [
        { role: "user", content: "abcd" },
        { role: "assistant", content: "efghijkl" },
      ],
      "gpt-4",
    );
    expect(total).toBe(14);
  });

  test("formatTokenCount adds separators", () => {
    expect(formatTokenCount(12000)).toBe("12,000");
  });

  test("calculateContextPercentage handles zero safely", () => {
    expect(calculateContextPercentage(10, 0)).toBe(0);
    expect(calculateContextPercentage(50, 200)).toBe(25);
  });

  test("usage color classes follow thresholds", () => {
    expect(getUsageColorClass(95)).toBe("text-danger");
    expect(getUsageColorClass(75)).toBe("text-warning");
    expect(getUsageColorClass(10)).toBe("text-success");
    expect(getUsageBackgroundClass(95)).toBe("bg-danger");
    expect(getUsageBackgroundClass(75)).toBe("bg-warning");
    expect(getUsageBackgroundClass(10)).toBe("bg-success");
  });

  test("formatCost uses precision rules", () => {
    expect(formatCost(0.005)).toBe("$0.0050");
    expect(formatCost(1.2)).toBe("$1.20");
  });
});
