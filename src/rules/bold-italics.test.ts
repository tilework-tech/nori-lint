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

describe("bold_italics fix", () => {
  test("strips **bold** to plain text", () => {
    const result = boldItalicsRule.fix!("Some **bold** content.");
    expect(result).toBe("Some bold content.");
  });

  test("strips *italic* to plain text", () => {
    const result = boldItalicsRule.fix!("Some *italic* content.");
    expect(result).toBe("Some italic content.");
  });

  test("strips __bold__ to plain text", () => {
    const result = boldItalicsRule.fix!("Some __bold__ content.");
    expect(result).toBe("Some bold content.");
  });

  test("strips ***bold italic*** to plain text", () => {
    const result = boldItalicsRule.fix!("Some ***bold italic*** content.");
    expect(result).toBe("Some bold italic content.");
  });

  test("preserves content inside fenced code blocks", () => {
    const input = "```\n**bold inside code**\n```";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe(input);
  });

  test("preserves bullet list items starting with *", () => {
    const input = "* Item one\n* Item two";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe(input);
  });

  test("preserves horizontal rule ***", () => {
    const input = "Some text\n***\nMore text";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe(input);
  });

  test("preserves inline code with bold markers", () => {
    const input = "Use `**bold**` for emphasis";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe(input);
  });

  test("fixes bold outside inline code, preserves inside", () => {
    const input = "**bold** and `**not bold**`";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe("bold and `**not bold**`");
  });

  test("fixes multiple violations across lines", () => {
    const input = "**bold** on line one\nplain text\n*italic* on line three";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe("bold on line one\nplain text\nitalic on line three");
  });

  test("returns unchanged content when no formatting present", () => {
    const input = "Plain text with no formatting.";
    const result = boldItalicsRule.fix!(input);
    expect(result).toBe(input);
  });

  test("fix is idempotent", () => {
    const input = "**bold** and *italic* and __underline__";
    const first = boldItalicsRule.fix!(input);
    const second = boldItalicsRule.fix!(first);
    expect(second).toBe(first);
  });

  test("fix resolves all violations detected by run", () => {
    const input = "**bold** and *italic* and __underline__";
    const fixed = boldItalicsRule.fix!(input);
    const violations = boldItalicsRule.run(fixed);
    expect(violations).toEqual([]);
  });
});
