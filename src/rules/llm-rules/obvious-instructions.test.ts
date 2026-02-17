import { obviousInstructionsRule } from "@/rules/llm-rules/obvious-instructions.js";

describe("obviousInstructionsRule", () => {
  test("has correct name", () => {
    expect(obviousInstructionsRule.name).toBe("obvious_instructions");
  });

  test("has non-empty description", () => {
    expect(obviousInstructionsRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(obviousInstructionsRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = obviousInstructionsRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when obvious instructions found", () => {
    const result = obviousInstructionsRule.evaluate("content", [
      {
        text: "read code changes carefully",
        reason: "LLMs already do this by default",
      },
      {
        text: "keep code modular where possible",
        reason: "generic best practice",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("read code changes carefully");
    expect(result!.message).toContain("keep code modular where possible");
  });

  test("returns violation with single obvious instruction", () => {
    const result = obviousInstructionsRule.evaluate("content", [
      { text: "write clean code", reason: "too vague to be actionable" },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("write clean code");
  });
});
