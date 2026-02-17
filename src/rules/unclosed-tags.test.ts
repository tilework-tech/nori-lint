import { unclosedTagsRule } from "@/rules/unclosed-tags.js";

describe("unclosed_tags rule", () => {
  test("returns empty when all tags closed", () => {
    const content = "<required>\nContent here.\n</required>";
    const result = unclosedTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when opening tag without closing", () => {
    const content = "<required>\nContent without closing tag.";
    const result = unclosedTagsRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message.toLowerCase()).toContain("required");
  });

  test("returns violation when closing tag without opening", () => {
    const content = "Content without opening tag.\n</required>";
    const result = unclosedTagsRule.run(content);
    expect(result.length).toBeGreaterThan(0);
  });

  test("returns empty with multiple different tags all closed", () => {
    const content =
      "<required>\nContent.\n</required>\n<example>\nMore content.\n</example>";
    const result = unclosedTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when one of multiple tags unclosed", () => {
    const content =
      "<required>\nContent.\n</required>\n<example>\nMore content.";
    const result = unclosedTagsRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message.toLowerCase()).toContain("example");
  });

  test("returns empty when no tags at all", () => {
    const content = "Just plain text with no tags.";
    const result = unclosedTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns empty with multiple instances of same tag all closed", () => {
    const content =
      "<required>\nFirst.\n</required>\n<required>\nSecond.\n</required>";
    const result = unclosedTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when counts do not match (2 open, 1 close)", () => {
    const content = "<required>\nFirst.\n<required>\nSecond.\n</required>";
    const result = unclosedTagsRule.run(content);
    expect(result.length).toBeGreaterThan(0);
  });

  test("returns empty for self-closing tags", () => {
    const content = "Line break here.<br/>\nHorizontal rule.<hr/>";
    const result = unclosedTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test('has correct name "unclosed_tags"', () => {
    expect(unclosedTagsRule.name).toBe("unclosed_tags");
  });

  test("has non-empty description", () => {
    expect(unclosedTagsRule.description.length).toBeGreaterThan(0);
  });
});
