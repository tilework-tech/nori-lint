import { describe, expect, test } from "vitest";

import { whenToUseRule } from "@/rules/when-to-use.js";

describe("when_to_use rule", () => {
  test('has correct name "when_to_use"', () => {
    expect(whenToUseRule.name).toBe("when_to_use");
  });

  test("has a description", () => {
    expect(whenToUseRule.description.length).toBeGreaterThan(0);
  });

  test("returns empty for file with no When to Use heading", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "Some content here.",
      "",
      "## How It Works",
      "",
      "Details.",
    ].join("\n");
    expect(whenToUseRule.run(input)).toEqual([]);
  });

  test("flags ## When to Use heading", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "## When to Use",
      "",
      "Use for any issue.",
    ].join("\n");
    const violations = whenToUseRule.run(input);
    expect(violations.length).toBe(1);
    expect(violations[0].line).toBe(6);
    expect(violations[0].snippet).toBe("## When to Use");
  });

  test("flags case-insensitive variations", () => {
    const variations = [
      "## When To Use",
      "# When to use",
      "### When to Use",
      "## when to use",
    ];
    for (const heading of variations) {
      const input = `---\nname: test\ndescription: Use when testing\n---\n\n${heading}\n\nContent.\n`;
      const violations = whenToUseRule.run(input);
      expect(violations.length).toBe(1);
      expect(violations[0].snippet).toBe(heading);
    }
  });

  test("does not flag When to Use inside code blocks", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "```markdown",
      "## When to Use",
      "```",
      "",
      "Real content.",
    ].join("\n");
    expect(whenToUseRule.run(input)).toEqual([]);
  });

  test("fix removes When to Use section up to next same-level heading", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "## Overview",
      "",
      "Some overview.",
      "",
      "## When to Use",
      "",
      "Use for any issue:",
      "- Bug fixes",
      "- Test failures",
      "",
      "## The Process",
      "",
      "Step 1.",
    ].join("\n");
    const fixed = whenToUseRule.fix!(input);
    expect(fixed).toContain("## Overview");
    expect(fixed).toContain("Some overview.");
    expect(fixed).not.toContain("When to Use");
    expect(fixed).not.toContain("Use for any issue");
    expect(fixed).not.toContain("Bug fixes");
    expect(fixed).toContain("## The Process");
    expect(fixed).toContain("Step 1.");
  });

  test("fix removes When to Use section at end of file", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "## Overview",
      "",
      "Some overview.",
      "",
      "## When to Use",
      "",
      "Use for any issue:",
      "- Bug fixes",
    ].join("\n");
    const fixed = whenToUseRule.fix!(input);
    expect(fixed).toContain("## Overview");
    expect(fixed).toContain("Some overview.");
    expect(fixed).not.toContain("When to Use");
    expect(fixed).not.toContain("Bug fixes");
  });

  test("fix resolves all violations detected by run", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "## When to Use",
      "",
      "Use for any issue.",
      "",
      "## Real Content",
      "",
      "Details.",
    ].join("\n");
    const fixed = whenToUseRule.fix!(input);
    expect(whenToUseRule.run(fixed)).toEqual([]);
  });

  test("fix is idempotent", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "## When to Use",
      "",
      "Use for any issue.",
      "",
      "## Real Content",
      "",
      "Details.",
    ].join("\n");
    const fixed = whenToUseRule.fix!(input);
    expect(whenToUseRule.fix!(fixed)).toBe(fixed);
  });

  test("fix preserves content before and after the section", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "Intro paragraph.",
      "",
      "## When to Use",
      "",
      "Stuff.",
      "",
      "## Next Section",
      "",
      "Important content.",
    ].join("\n");
    const fixed = whenToUseRule.fix!(input);
    expect(fixed).toContain("Intro paragraph.");
    expect(fixed).toContain("## Next Section");
    expect(fixed).toContain("Important content.");
  });

  test("fix does not leave triple blank lines", () => {
    const input = [
      "---",
      "name: my-skill",
      "description: Use when testing things",
      "---",
      "",
      "## Before",
      "",
      "Content.",
      "",
      "## When to Use",
      "",
      "Stuff.",
      "",
      "## After",
      "",
      "More content.",
    ].join("\n");
    const fixed = whenToUseRule.fix!(input);
    expect(fixed).not.toMatch(/\n{3,}/);
  });
});
