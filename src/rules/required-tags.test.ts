import { requiredTagsRule } from "@/rules/required-tags.js";

describe("required_tags rule", () => {
  test("returns empty when <required> and </required> both present", () => {
    const content = "<required>\nSome important content.\n</required>";
    const result = requiredTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test("returns violation when no required tags", () => {
    const content = "Just some plain text without any required tags.";
    const result = requiredTagsRule.run(content);
    expect(result.length).toBeGreaterThan(0);
    expect(result[0].message.toLowerCase()).toContain("required");
  });

  test("returns empty when multiple required pairs", () => {
    const content =
      "<required>\nFirst block.\n</required>\n\n<required>\nSecond block.\n</required>";
    const result = requiredTagsRule.run(content);
    expect(result).toEqual([]);
  });

  test('has correct name "required_tags"', () => {
    expect(requiredTagsRule.name).toBe("required_tags");
  });

  test("has non-empty description", () => {
    expect(requiredTagsRule.description.length).toBeGreaterThan(0);
  });
});
