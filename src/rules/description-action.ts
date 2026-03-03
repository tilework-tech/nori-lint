import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

export const descriptionActionRule: Rule = {
  name: "description_action",
  description:
    "Use when linting SKILL.md files to ensure description fields indicate when to use the skill, not what it does",
  run: (input): Array<RuleViolation> => {
    const lines = input.split("\n");

    if (lines.length === 0 || lines[0].trim() !== "---") {
      return [{ message: "Missing frontmatter with description field" }];
    }

    let closingIdx = -1;
    for (let i = 1; i < lines.length; i++) {
      if (lines[i].trim() === "---") {
        closingIdx = i;
        break;
      }
    }

    if (closingIdx === -1) {
      return [{ message: "Missing frontmatter with description field" }];
    }

    let descriptionValue: string | null = null;
    let descriptionLine: number | null = null;

    for (let i = 1; i < closingIdx; i++) {
      const match = lines[i].match(/^description:\s*(.*)/);
      if (match) {
        descriptionLine = i + 1;
        descriptionValue = match[1].trim();
        break;
      }
    }

    if (descriptionValue === null || descriptionValue === "") {
      return [
        {
          message: "Missing frontmatter with description field",
          line: descriptionLine ?? undefined,
        },
      ];
    }

    // Strip surrounding quotes
    if (
      (descriptionValue.startsWith('"') && descriptionValue.endsWith('"')) ||
      (descriptionValue.startsWith("'") && descriptionValue.endsWith("'"))
    ) {
      descriptionValue = descriptionValue.slice(1, -1);
    }

    if (!descriptionValue.toLowerCase().startsWith("use ")) {
      return [
        {
          message:
            'Description should indicate when to use the skill (e.g. "Use when ..."), not what it does',
          line: descriptionLine ?? undefined,
          snippet: descriptionValue,
        },
      ];
    }

    return [];
  },
};
