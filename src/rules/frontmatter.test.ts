import { frontmatterRule } from "@/rules/frontmatter.js";

describe("frontmatter rule", () => {
  test("returns empty for valid minimal frontmatter", () => {
    const content =
      "---\nname: my-skill\ndescription: A skill that does things.\n---\n\nBody content here.";
    const result = frontmatterRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty for valid frontmatter with all optional fields", () => {
    const content = [
      "---",
      "name: my-skill",
      "description: A skill that does things.",
      "license: MIT",
      "compatibility: Requires git and docker",
      "metadata:",
      "  author: example-org",
      '  version: "1.0"',
      "allowed-tools: Bash(git:*) Read",
      "---",
      "",
      "Body content.",
    ].join("\n");
    const result = frontmatterRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when no frontmatter at all", () => {
    const content = "Just some markdown content without frontmatter.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("frontmatter");
  });

  test("returns violation when opening --- but no closing ---", () => {
    const content =
      "---\nname: my-skill\ndescription: Does things.\nNo closing delimiter.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("frontmatter");
  });

  test("returns violations for empty frontmatter (no fields)", () => {
    const content = "---\n---\n\nBody content.";
    const result = frontmatterRule.run(content);
    const messages = result.map((v) => v.message).join(" ");
    expect(messages).toContain("name");
    expect(messages).toContain("description");
  });

  test("returns violation when name is missing", () => {
    const content = "---\ndescription: Does things.\n---\n\nBody.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("name");
  });

  test("returns violation when description is missing", () => {
    const content = "---\nname: my-skill\n---\n\nBody.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("description");
  });

  test("returns violation when name is empty string", () => {
    const content = '---\nname: ""\ndescription: Does things.\n---\n\nBody.';
    const result = frontmatterRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("name");
  });

  test("returns violation when description is empty string", () => {
    const content = '---\nname: my-skill\ndescription: ""\n---\n\nBody.';
    const result = frontmatterRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message).toContain("description");
  });

  test("returns violation when description exceeds 1024 characters", () => {
    const longDesc = "a".repeat(1025);
    const content = `---\nname: my-skill\ndescription: ${longDesc}\n---\n\nBody.`;
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("1024");
  });

  test("returns empty when description is exactly 1024 characters", () => {
    const desc = "a".repeat(1024);
    const content = `---\nname: my-skill\ndescription: ${desc}\n---\n\nBody.`;
    const result = frontmatterRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when compatibility exceeds 500 characters", () => {
    const longCompat = "a".repeat(501);
    const content = `---\nname: my-skill\ndescription: Does things.\ncompatibility: ${longCompat}\n---\n\nBody.`;
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("500");
  });

  test("returns empty when compatibility is exactly 500 characters", () => {
    const compat = "a".repeat(500);
    const content = `---\nname: my-skill\ndescription: Does things.\ncompatibility: ${compat}\n---\n\nBody.`;
    const result = frontmatterRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when metadata is a scalar string", () => {
    const content =
      "---\nname: my-skill\ndescription: Does things.\nmetadata: just-a-string\n---\n\nBody.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("metadata");
  });

  test("returns violation when metadata is an array", () => {
    const content =
      "---\nname: my-skill\ndescription: Does things.\nmetadata:\n  - item1\n  - item2\n---\n\nBody.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("metadata");
  });

  test("returns empty when metadata is a valid map", () => {
    const content =
      '---\nname: my-skill\ndescription: Does things.\nmetadata:\n  author: me\n  version: "1.0"\n---\n\nBody.';
    const result = frontmatterRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation for unknown fields", () => {
    const content =
      "---\nname: my-skill\ndescription: Does things.\nunknown-field: value\n---\n\nBody.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBe(1);
    expect(result[0].message).toContain("unknown-field");
  });

  test("returns violation for invalid YAML in frontmatter", () => {
    const content = "---\nname: my-skill\n  bad-indent: oops\n---\n\nBody.";
    const result = frontmatterRule.run(content);
    expect(result.length).toBeGreaterThan(0);
  });
});
