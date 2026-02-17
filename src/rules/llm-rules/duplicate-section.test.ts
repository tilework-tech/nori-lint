import { duplicateSectionRule } from "@/rules/llm-rules/duplicate-section.js";

describe("duplicateSectionRule", () => {
  test("has correct name", () => {
    expect(duplicateSectionRule.name).toBe("duplicate_section");
  });

  test("has non-empty description", () => {
    expect(duplicateSectionRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(duplicateSectionRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = duplicateSectionRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when duplicate sections found", () => {
    const result = duplicateSectionRule.evaluate("content", [
      {
        text: "Setup / Installation",
        reason: "These sections cover the same topic",
      },
      { text: "Red Flags / Common Mistakes", reason: "Overlapping content" },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Setup / Installation");
    expect(result!.message).toContain("Red Flags / Common Mistakes");
  });

  test("returns violation with single duplicate pair", () => {
    const result = duplicateSectionRule.evaluate("content", [
      { text: "Overview / Introduction", reason: "Both introduce the topic" },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Overview / Introduction");
  });
});
