import { consecutiveBlankLinesRule } from "@/rules/consecutive-blank-lines.js";

describe("consecutive_blank_lines rule", () => {
  test('has correct name "consecutive_blank_lines"', () => {
    expect(consecutiveBlankLinesRule.name).toBe("consecutive_blank_lines");
  });

  test("has non-empty description", () => {
    expect(consecutiveBlankLinesRule.description.length).toBeGreaterThan(0);
  });

  // --- Detection (run) ---

  test("returns empty for content with no consecutive blank lines", () => {
    const result = consecutiveBlankLinesRule.run(
      "line one\n\nline two\n\nline three",
    );
    expect(result).toEqual([]);
  });

  test("returns empty for content with single blank lines between paragraphs", () => {
    const result = consecutiveBlankLinesRule.run(
      "paragraph one\n\nparagraph two\n\nparagraph three",
    );
    expect(result).toEqual([]);
  });

  test("flags two consecutive blank lines (three newlines in a row)", () => {
    const result = consecutiveBlankLinesRule.run("line one\n\n\nline two");
    expect(result.length).toBe(1);
    expect(result[0].line).toBeDefined();
  });

  test("flags three consecutive blank lines", () => {
    const result = consecutiveBlankLinesRule.run("line one\n\n\n\nline two");
    expect(result.length).toBe(1);
  });

  test("reports correct line number for violation", () => {
    const result = consecutiveBlankLinesRule.run(
      "line one\nline two\n\n\nline five",
    );
    expect(result.length).toBe(1);
    // The second blank line (the extra one) is on line 4 (1-indexed)
    expect(result[0].line).toBe(4);
  });

  test("flags multiple separate runs of consecutive blank lines", () => {
    const result = consecutiveBlankLinesRule.run("one\n\n\ntwo\n\n\nthree");
    expect(result.length).toBe(2);
  });

  test("returns empty for single-line content", () => {
    const result = consecutiveBlankLinesRule.run("just one line");
    expect(result).toEqual([]);
  });

  test("returns empty for empty string", () => {
    const result = consecutiveBlankLinesRule.run("");
    expect(result).toEqual([]);
  });

  // --- Fix ---

  test("collapses three consecutive newlines to two", () => {
    const result = consecutiveBlankLinesRule.fix!("one\n\n\ntwo");
    expect(result).toBe("one\n\ntwo");
  });

  test("collapses four consecutive newlines to two", () => {
    const result = consecutiveBlankLinesRule.fix!("one\n\n\n\ntwo");
    expect(result).toBe("one\n\ntwo");
  });

  test("collapses multiple separate runs", () => {
    const result = consecutiveBlankLinesRule.fix!("one\n\n\ntwo\n\n\nthree");
    expect(result).toBe("one\n\ntwo\n\nthree");
  });

  test("preserves single blank lines", () => {
    const input = "one\n\ntwo\n\nthree";
    const result = consecutiveBlankLinesRule.fix!(input);
    expect(result).toBe(input);
  });

  test("fix is idempotent", () => {
    const input = "one\n\n\n\ntwo\n\n\nthree";
    const first = consecutiveBlankLinesRule.fix!(input);
    const second = consecutiveBlankLinesRule.fix!(first);
    expect(second).toBe(first);
  });

  test("fix resolves all violations detected by run", () => {
    const input = "one\n\n\ntwo\n\n\n\nthree";
    const fixed = consecutiveBlankLinesRule.fix!(input);
    const violations = consecutiveBlankLinesRule.run(fixed);
    expect(violations).toEqual([]);
  });
});
