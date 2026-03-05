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

describe("redundant_title fix", () => {
  test("removes ATX heading at start of file", () => {
    const result = redundantTitleRule.fix!("# My Title\n\nSome body text.");
    expect(result).toBe("Some body text.");
  });

  test("removes ATX heading after frontmatter", () => {
    const input = "---\ntitle: Test\n---\n# My Title\n\nBody text.";
    const result = redundantTitleRule.fix!(input);
    expect(result).toBe("---\ntitle: Test\n---\nBody text.");
  });

  test("removes setext h1 heading (text + === underline)", () => {
    const input = "My Title\n========\n\nBody text.";
    const result = redundantTitleRule.fix!(input);
    expect(result).toBe("Body text.");
  });

  test("removes setext h2 heading (text + --- underline)", () => {
    const input = "My Title\n--------\n\nBody text.";
    const result = redundantTitleRule.fix!(input);
    expect(result).toBe("Body text.");
  });

  test("removes setext heading after frontmatter", () => {
    const input = "---\ntitle: Test\n---\nMy Title\n========\n\nBody text.";
    const result = redundantTitleRule.fix!(input);
    expect(result).toBe("---\ntitle: Test\n---\nBody text.");
  });

  test("returns unchanged content when no heading present", () => {
    const input = "Some body text.\n\nMore text.";
    const result = redundantTitleRule.fix!(input);
    expect(result).toBe(input);
  });

  test("returns unchanged content for empty file", () => {
    const result = redundantTitleRule.fix!("");
    expect(result).toBe("");
  });

  test("fix is idempotent", () => {
    const input = "# My Title\n\nSome body text.";
    const first = redundantTitleRule.fix!(input);
    const second = redundantTitleRule.fix!(first);
    expect(second).toBe(first);
  });

  test("fix resolves all violations detected by run", () => {
    const input = "# My Title\n\nSome body text.";
    const fixed = redundantTitleRule.fix!(input);
    const violations = redundantTitleRule.run(fixed);
    expect(violations).toEqual([]);
  });
});
