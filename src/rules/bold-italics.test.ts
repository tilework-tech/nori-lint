import { boldItalicsRule } from "@/rules/bold-italics.js";

describe("bold_italics rule", () => {
  test("returns empty for clean content", () => {
    const result = boldItalicsRule.run(
      "This is plain text with no formatting.",
    );
    expect(result).toEqual([]);
  });

  test("flags **bold** (double star)", () => {
    const result = boldItalicsRule.run("This has **bold** text.");
    expect(result.length).toBeGreaterThan(0);
  });

  test("flags *italic* (single star)", () => {
    const result = boldItalicsRule.run("This has *italic* text.");
    expect(result.length).toBeGreaterThan(0);
  });

  test("flags ***bold italic*** (triple star)", () => {
    const result = boldItalicsRule.run("This has ***bold italic*** text.");
    expect(result.length).toBeGreaterThan(0);
  });

  test("flags __bold__ (double underscore)", () => {
    const result = boldItalicsRule.run("This has __bold__ text.");
    expect(result.length).toBeGreaterThan(0);
  });

  test('does not flag bullet list items starting with "* "', () => {
    const content = "* Item one\n* Item two\n* Item three";
    const result = boldItalicsRule.run(content);
    expect(result).toEqual([]);
  });

  test("does not flag horizontal rule (*** alone on a line)", () => {
    const result = boldItalicsRule.run("Some text\n***\nMore text");
    expect(result).toEqual([]);
  });

  test("does not flag horizontal rule with spaces (* * *)", () => {
    const result = boldItalicsRule.run("Some text\n* * *\nMore text");
    expect(result).toEqual([]);
  });

  test("reports multiple violations across lines", () => {
    const content = "**bold** on line one\nplain text\n*italic* on line three";
    const result = boldItalicsRule.run(content);
    expect(result.length).toBeGreaterThanOrEqual(2);
  });

  test("reports multiple violations on same line", () => {
    const content = "**bold** and *italic*";
    const result = boldItalicsRule.run(content);
    expect(result.length).toBeGreaterThanOrEqual(2);
  });

  test("does not flag single asterisk alone", () => {
    const result = boldItalicsRule.run("Use * for multiplication");
    expect(result).toEqual([]);
  });

  test("does not flag double asterisk alone", () => {
    const result = boldItalicsRule.run("Use ** for exponentiation");
    expect(result).toEqual([]);
  });

  test("does not flag bold inside fenced code block", () => {
    const content = "```\n**bold inside code**\n```";
    const result = boldItalicsRule.run(content);
    expect(result).toEqual([]);
  });

  test("does not flag bold inside fenced code block with language tag", () => {
    const content = "```python\n**bold inside code**\n```";
    const result = boldItalicsRule.run(content);
    expect(result).toEqual([]);
  });

  test("flags bold outside code block but not inside", () => {
    const content =
      "**bold outside**\n```\n**bold inside code**\n```\nplain text";
    const result = boldItalicsRule.run(content);
    expect(result.length).toBe(1);
  });

  test("does not flag bold inside inline code", () => {
    const content = "Use `**bold**` for emphasis";
    const result = boldItalicsRule.run(content);
    expect(result).toEqual([]);
  });

  test("flags bold outside inline code", () => {
    const content = "**bold** and `**not bold**`";
    const result = boldItalicsRule.run(content);
    expect(result.length).toBe(1);
  });

  test('has correct name "bold_italics"', () => {
    expect(boldItalicsRule.name).toBe("bold_italics");
  });

  test("has non-empty description", () => {
    expect(boldItalicsRule.description.length).toBeGreaterThan(0);
  });
});
