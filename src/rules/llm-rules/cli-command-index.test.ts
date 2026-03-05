import { cliCommandIndexRule } from "@/rules/llm-rules/cli-command-index.js";

describe("cliCommandIndexRule", () => {
  test("has correct name", () => {
    expect(cliCommandIndexRule.name).toBe("cli_command_index");
  });

  test("has non-empty description", () => {
    expect(cliCommandIndexRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(cliCommandIndexRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = cliCommandIndexRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when command index found", () => {
    const result = cliCommandIndexRule.evaluate("content", [
      {
        text: "foo --bar  Use this\nfoo --baz  Use this",
        reason: "tabular CLI index",
      },
      {
        text: "Available commands:\nfoo\nfoo bar",
        reason: "bare command list",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("foo --bar");
    expect(result!.message).toContain("Available commands");
  });

  test("returns violation with single index", () => {
    const result = cliCommandIndexRule.evaluate("content", [
      { text: "git status\ngit add\ngit commit", reason: "bare list" },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("git status");
  });
});
