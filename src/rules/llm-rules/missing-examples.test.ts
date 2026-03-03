import { missingExamplesRule } from "@/rules/llm-rules/missing-examples.js";

describe("missing_examples rule", () => {
  test('has correct name "missing_examples"', () => {
    expect(missingExamplesRule.name).toBe("missing_examples");
  });

  test("has non-empty description", () => {
    expect(missingExamplesRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(missingExamplesRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = missingExamplesRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when examples are missing", () => {
    const result = missingExamplesRule.evaluate("content", [
      {
        text: "Steps should be actionable and concrete",
        reason:
          "This describes expected behavior without showing what actionable looks like",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain(
      "Steps should be actionable and concrete",
    );
  });

  test("returns violation with multiple missing example locations", () => {
    const result = missingExamplesRule.evaluate("content", [
      {
        text: "Use kebab-case naming",
        reason: "Naming convention stated without showing good vs bad examples",
      },
      {
        text: "Commit messages should be descriptive",
        reason:
          "Expectation set without concrete examples of good vs bad messages",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Use kebab-case naming");
    expect(result!.message).toContain("Commit messages should be descriptive");
  });
});
