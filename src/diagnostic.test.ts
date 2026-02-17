import { fromViolation } from "@/diagnostic.js";

describe("fromViolation", () => {
  test("creates a LintDiagnostic from a violation with all fields", () => {
    const violation = {
      message: "Something is wrong",
      line: 42,
      snippet: "the offending line",
    };

    const result = fromViolation(violation, "my-rule", "/path/to/file.md");

    expect(result).toEqual({
      rule: "my-rule",
      file: "/path/to/file.md",
      line: 42,
      snippet: "the offending line",
      message: "Something is wrong",
    });
  });

  test("creates a LintDiagnostic with null line and snippet when not provided", () => {
    const violation = {
      message: "Only a message",
    };

    const result = fromViolation(violation, "another-rule", "/some/file.md");

    expect(result.line).toBeNull();
    expect(result.snippet).toBeNull();
    expect(result.message).toBe("Only a message");
  });

  test("rule and file come from function args, not the violation", () => {
    const violation = {
      message: "test message",
      line: 1,
      snippet: "snippet",
    };

    const result = fromViolation(
      violation,
      "expected-rule",
      "/expected/path.md",
    );

    expect(result.rule).toBe("expected-rule");
    expect(result.file).toBe("/expected/path.md");
  });
});
