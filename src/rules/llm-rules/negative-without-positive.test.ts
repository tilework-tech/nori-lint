import { negativeWithoutPositiveRule } from "@/rules/llm-rules/negative-without-positive.js";

describe("negativeWithoutPositiveRule", () => {
  test("has correct name", () => {
    expect(negativeWithoutPositiveRule.name).toBe("negative_without_positive");
  });

  test("has non-empty description", () => {
    expect(negativeWithoutPositiveRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(negativeWithoutPositiveRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = negativeWithoutPositiveRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation with offending text and suggestion", () => {
    const result = negativeWithoutPositiveRule.evaluate("content", [
      {
        text: "Don't use global variables.",
        reason: "Should suggest passing dependencies as function parameters",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Don't use global variables.");
    expect(result!.message).toContain(
      "Should suggest passing dependencies as function parameters",
    );
  });

  test("returns violation with multiple negatives", () => {
    const result = negativeWithoutPositiveRule.evaluate("content", [
      {
        text: "Don't use global variables.",
        reason: "Should suggest passing dependencies as function parameters",
      },
      {
        text: "Never hardcode secrets.",
        reason: "Should suggest using environment variables or secret managers",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Don't use global variables.");
    expect(result!.message).toContain("Never hardcode secrets.");
  });
});
