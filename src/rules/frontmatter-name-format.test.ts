import { frontmatterNameFormatRule } from "@/rules/frontmatter-name-format.js";

describe("frontmatter_name_format rule", () => {
  test("returns empty for valid lowercase hyphenated name", () => {
    const content =
      "---\nname: my-skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty for single word lowercase name", () => {
    const content = "---\nname: skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty for name with numbers", () => {
    const content =
      "---\nname: skill-v2\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty for name at exactly 64 characters", () => {
    const name = "a".repeat(64);
    const content = `---\nname: ${name}\ndescription: Does things.\n---\n\nBody.`;
    const result = frontmatterNameFormatRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation for name exceeding 64 characters", () => {
    const name = "a".repeat(65);
    const content = `---\nname: ${name}\ndescription: Does things.\n---\n\nBody.`;
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("64");
  });

  test("returns exactly one violation for name with uppercase letters", () => {
    const content =
      "---\nname: My-Skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].snippet).toBe("My-Skill");
  });

  test("returns exactly one violation for name with spaces", () => {
    const content =
      "---\nname: my skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns exactly one violation for name starting with hyphen", () => {
    const content =
      "---\nname: -my-skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns exactly one violation for name ending with hyphen", () => {
    const content =
      "---\nname: my-skill-\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns exactly one violation for name with consecutive hyphens", () => {
    const content =
      "---\nname: my--skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns exactly one violation for name with underscores", () => {
    const content =
      "---\nname: my_skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns exactly one violation for name with special characters", () => {
    const content =
      "---\nname: my@skill!\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
  });

  test("returns exactly one violation for name with multiple problems (uppercase and spaces)", () => {
    const content =
      "---\nname: Finishing a Development Branch\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].snippet).toBe("Finishing a Development Branch");
  });

  test("returns empty when no frontmatter (skips gracefully)", () => {
    const content = "No frontmatter here.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty when frontmatter has no name field (skips gracefully)", () => {
    const content = "---\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result).toEqual([]);
  });

  test('has correct name "frontmatter_name_format"', () => {
    expect(frontmatterNameFormatRule.name).toBe("frontmatter_name_format");
  });

  test("has non-empty description", () => {
    expect(frontmatterNameFormatRule.description.length).toBeGreaterThan(0);
  });
});
