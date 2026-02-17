import type { Rule } from "@/rules/index.js";

export const requiredTagsRule: Rule = {
  name: "required_tags",
  description:
    "Checks that SKILL.md files contain at least one <required> block",
  run: (input) => {
    const hasOpen = input.includes("<required>");
    const hasClose = input.includes("</required>");
    if (hasOpen && hasClose) {
      return [];
    }
    return [{ message: "File is missing a <required> block" }];
  },
};
