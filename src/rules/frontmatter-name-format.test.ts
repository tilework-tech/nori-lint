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

  test("returns violation for name with uppercase letters", () => {
    const content =
      "---\nname: My-Skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message.toLowerCase()).toContain("lowercase");
  });

  test("returns violation for name with spaces", () => {
    const content =
      "---\nname: my skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
  });

  test("returns violation for name starting with hyphen", () => {
    const content =
      "---\nname: -my-skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("start");
  });

  test("returns violation for name ending with hyphen", () => {
    const content =
      "---\nname: my-skill-\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("end");
  });

  test("returns violation for name with consecutive hyphens", () => {
    const content =
      "---\nname: my--skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("consecutive");
  });

  test("returns violation for name with underscores", () => {
    const content =
      "---\nname: my_skill\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
  });

  test("returns violation for name with special characters", () => {
    const content =
      "---\nname: my@skill!\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterNameFormatRule.run(content);
    expect(result.length).toBeGreaterThan(0);
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
});
