import { trailingWhitespaceRule } from "@/rules/trailing-whitespace.js";

describe("trailing_whitespace rule", () => {
  test('has correct name "trailing_whitespace"', () => {
    expect(trailingWhitespaceRule.name).toBe("trailing_whitespace");
  });

  test("has non-empty description", () => {
    expect(trailingWhitespaceRule.description.length).toBeGreaterThan(0);
  });

  // --- Detection (run) ---

  test("returns empty for content with no trailing whitespace", () => {
    const result = trailingWhitespaceRule.run("line one\nline two\nline three");
    expect(result).toEqual([]);
  });

  test("flags line with trailing spaces", () => {
    const result = trailingWhitespaceRule.run("line one   \nline two");
    expect(result.length).toBe(1);
    expect(result[0].line).toBe(1);
  });

  test("flags line with trailing tab", () => {
    const result = trailingWhitespaceRule.run("line one\t\nline two");
    expect(result.length).toBe(1);
    expect(result[0].line).toBe(1);
  });

  test("flags line with mixed trailing whitespace", () => {
    const result = trailingWhitespaceRule.run("line one \t \nline two");
    expect(result.length).toBe(1);
  });

  test("flags multiple lines with trailing whitespace", () => {
    const result = trailingWhitespaceRule.run("one  \ntwo\nthree  ");
    expect(result.length).toBe(2);
    expect(result[0].line).toBe(1);
    expect(result[1].line).toBe(3);
  });

  test("does not flag blank lines", () => {
    const result = trailingWhitespaceRule.run("one\n\ntwo");
    expect(result).toEqual([]);
  });

  test("returns empty for empty string", () => {
    const result = trailingWhitespaceRule.run("");
    expect(result).toEqual([]);
  });

  test("includes snippet in violation", () => {
    const result = trailingWhitespaceRule.run("hello   \nworld");
    expect(result.length).toBe(1);
    expect(result[0].snippet).toBeDefined();
  });

  // --- Fix ---

  test("strips trailing spaces from a line", () => {
    const result = trailingWhitespaceRule.fix!("hello   \nworld");
    expect(result).toBe("hello\nworld");
  });

  test("strips trailing tabs from a line", () => {
    const result = trailingWhitespaceRule.fix!("hello\t\nworld");
    expect(result).toBe("hello\nworld");
  });

  test("strips trailing whitespace from multiple lines", () => {
    const result = trailingWhitespaceRule.fix!("one  \ntwo\t\nthree   ");
    expect(result).toBe("one\ntwo\nthree");
  });

  test("preserves leading whitespace", () => {
    const result = trailingWhitespaceRule.fix!("  indented  \n    more  ");
    expect(result).toBe("  indented\n    more");
  });

  test("preserves lines without trailing whitespace", () => {
    const input = "clean\nlines\nhere";
    const result = trailingWhitespaceRule.fix!(input);
    expect(result).toBe(input);
  });

  test("fix is idempotent", () => {
    const input = "one  \ntwo\t\nthree   ";
    const first = trailingWhitespaceRule.fix!(input);
    const second = trailingWhitespaceRule.fix!(first);
    expect(second).toBe(first);
  });

  test("fix resolves all violations detected by run", () => {
    const input = "one  \ntwo\t\nthree   ";
    const fixed = trailingWhitespaceRule.fix!(input);
    const violations = trailingWhitespaceRule.run(fixed);
    expect(violations).toEqual([]);
  });
});
