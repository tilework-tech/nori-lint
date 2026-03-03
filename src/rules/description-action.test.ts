import { descriptionActionRule } from "@/rules/description-action.js";

describe("descriptionActionRule", () => {
  test("passes when description starts with 'Use when'", () => {
    const input = `---
name: My Skill
description: Use when encountering any bug or test failure
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("passes when description starts with 'Use this when'", () => {
    const input = `---
name: My Skill
description: Use this when you have finished making code changes
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("passes when description starts with 'Use after'", () => {
    const input = `---
name: My Skill
description: Use after TDD is finished, to review and clean the testing additions
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("passes when description starts with 'Use this skill when'", () => {
    const input = `---
name: My Skill
description: Use this skill when faced with a difficult debugging task
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("fails when description starts with 'Describes'", () => {
    const input = `---
name: My Skill
description: Describes how to use abilities. Read before any conversation.
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].message).toContain("when to use");
    expect(result[0].snippet).toBe(
      "Describes how to use abilities. Read before any conversation.",
    );
    expect(result[0].line).toBe(3);
  });

  test("fails when description is a noun phrase", () => {
    const input = `---
name: My Skill
description: A tool for debugging complex issues
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].message).toContain("when to use");
  });

  test("fails when there is no frontmatter", () => {
    const input = `# My Skill

Some content without frontmatter.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].message).toContain("description");
  });

  test("fails when frontmatter has no description field", () => {
    const input = `---
name: My Skill
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].message).toContain("description");
  });

  test("fails when description is empty", () => {
    const input = `---
name: My Skill
description:
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].message).toContain("description");
  });

  test("passes with case-insensitive 'use when'", () => {
    const input = `---
name: My Skill
description: use when you need to debug something
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("handles quoted description values", () => {
    const input = `---
name: My Skill
description: "Use when implementing new features"
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("handles single-quoted description values", () => {
    const input = `---
name: My Skill
description: 'Use when implementing new features'
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toEqual([]);
  });

  test("fails when frontmatter is unclosed", () => {
    const input = `---
name: My Skill
description: Use when testing things

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].message).toContain("description");
  });

  test("reports correct line number for description", () => {
    const input = `---
name: My Skill
version: 1.0
description: Provides debugging tools
---

Some content here.
`;
    const result = descriptionActionRule.run(input);
    expect(result).toHaveLength(1);
    expect(result[0].line).toBe(4);
  });
});
