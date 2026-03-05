import { skillExampleXmlTagsRule } from "@/rules/llm-rules/skill-example-xml-tags.js";

describe("skillExampleXmlTagsRule", () => {
  test("has correct name", () => {
    expect(skillExampleXmlTagsRule.name).toBe("skill_example_xml_tags");
  });

  test("has non-empty description", () => {
    expect(skillExampleXmlTagsRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(skillExampleXmlTagsRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = skillExampleXmlTagsRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when unwrapped example found", () => {
    const result = skillExampleXmlTagsRule.evaluate("content", [
      {
        text: "I will fix the bug now.",
        reason: "Should be wrapped in <good_example> tags",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("I will fix the bug now.");
    expect(result!.message).toContain(
      "Should be wrapped in <good_example> tags",
    );
  });

  test("returns violation with multiple unwrapped examples", () => {
    const result = skillExampleXmlTagsRule.evaluate("content", [
      {
        text: "Sure, I can do that for you!",
        reason: "Should be wrapped in <bad_example> tags",
      },
      {
        text: "Let me investigate the root cause first.",
        reason: "Should be wrapped in <good_example> tags",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Sure, I can do that for you!");
    expect(result!.message).toContain(
      "Let me investigate the root cause first.",
    );
  });
});
