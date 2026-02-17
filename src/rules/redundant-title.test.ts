import { redundantTitleRule } from "@/rules/redundant-title.js";

describe("redundant_title rule", () => {
  test("returns violation for ATX heading at start", () => {
    const result = redundantTitleRule.run("# My Title\n\nSome body text.");
    expect(result.length).toBe(1);
    expect(result[0].line).toBe(1);
    expect(result[0].snippet).toContain("# My Title");
  });

  test("returns empty for body text at start", () => {
    const result = redundantTitleRule.run("Some body text.\n\nMore text.");
    expect(result).toEqual([]);
  });

  test("returns violation for heading after frontmatter", () => {
    const content = "---\ntitle: Test\n---\n# My Title\n\nBody text.";
    const result = redundantTitleRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].line).toBe(4);
  });

  test("returns empty for body text after frontmatter", () => {
    const content = "---\ntitle: Test\n---\nSome body text.";
    const result = redundantTitleRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty for empty file", () => {
    const result = redundantTitleRule.run("");
    expect(result).toEqual([]);
  });

  test("returns empty for whitespace-only file", () => {
    const result = redundantTitleRule.run("   \n  \n   ");
    expect(result).toEqual([]);
  });

  test("returns violation for h2 heading", () => {
    const result = redundantTitleRule.run("## Subtitle\n\nBody text.");
    expect(result.length).toBe(1);
  });

  test("returns violation for h3 heading", () => {
    const result = redundantTitleRule.run("### Section\n\nBody text.");
    expect(result.length).toBe(1);
  });

  test("returns violation for setext h1 (text + === underline)", () => {
    const content = "My Title\n========\n\nBody text.";
    const result = redundantTitleRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns violation for setext h2 (text + --- underline)", () => {
    const content = "My Title\n--------\n\nBody text.";
    const result = redundantTitleRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns violation for setext heading after frontmatter", () => {
    const content = "---\ntitle: Test\n---\nMy Title\n========\n\nBody text.";
    const result = redundantTitleRule.run(content);
    expect(result.length).toBe(1);
  });

  test("does not confuse frontmatter delimiter with setext", () => {
    const content = "---\ntitle: Test\n---\nBody text here.";
    const result = redundantTitleRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty for no content after frontmatter", () => {
    const content = "---\ntitle: Test\n---";
    const result = redundantTitleRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty when heading appears later in body", () => {
    const content = "Some intro text.\n\n# Heading Later";
    const result = redundantTitleRule.run(content);
    expect(result).toEqual([]);
  });

  test('has correct name "redundant_title"', () => {
    expect(redundantTitleRule.name).toBe("redundant_title");
  });

  test("has non-empty description", () => {
    expect(redundantTitleRule.description.length).toBeGreaterThan(0);
  });
});
