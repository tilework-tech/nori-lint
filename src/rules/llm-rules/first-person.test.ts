import { firstPersonRule } from "@/rules/llm-rules/first-person.js";

describe("firstPersonRule", () => {
  test("has correct name", () => {
    expect(firstPersonRule.name).toBe("first_person");
  });

  test("has non-empty description", () => {
    expect(firstPersonRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(firstPersonRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = firstPersonRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation with offending text and suggested replacement", () => {
    const result = firstPersonRule.evaluate("content", [
      {
        text: "Ask the user to answer the question",
        reason: "Ask me to answer the question",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Ask the user to answer the question");
    expect(result!.message).toContain("Ask me to answer the question");
  });

  test("returns violation with multiple first-person instances", () => {
    const result = firstPersonRule.evaluate("content", [
      {
        text: "Tell the user about the feature",
        reason: "Tell me about the feature",
      },
      { text: "Show the user the results", reason: "Show me the results" },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Tell the user about the feature");
    expect(result!.message).toContain("Show the user the results");
  });
});
