import { unexplainedUrlRule } from "@/rules/llm-rules/unexplained-url.js";

describe("unexplainedUrlRule", () => {
  test("has correct name", () => {
    expect(unexplainedUrlRule.name).toBe("unexplained_url");
  });

  test("has non-empty description", () => {
    expect(unexplainedUrlRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(unexplainedUrlRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = unexplainedUrlRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when unexplained URL found", () => {
    const result = unexplainedUrlRule.evaluate("content", [
      {
        text: "https://raw.githubusercontent.com/org/repo/abc123/path/file.md",
        reason: "URL lacks context about what it points to",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain(
      "https://raw.githubusercontent.com/org/repo/abc123/path/file.md",
    );
  });

  test("returns violation with multiple URLs", () => {
    const result = unexplainedUrlRule.evaluate("content", [
      {
        text: "https://example.com/api/v2/docs",
        reason: "bare URL without description",
      },
      {
        text: "https://cdn.example.com/assets/config.yaml",
        reason: "no explanation of purpose",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("https://example.com/api/v2/docs");
    expect(result!.message).toContain(
      "https://cdn.example.com/assets/config.yaml",
    );
  });
});
