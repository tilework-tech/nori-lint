import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

export const trailingWhitespaceRule: Rule = {
  name: "trailing_whitespace",
  description:
    "Checks that SKILL.md files do not have trailing whitespace on lines",
  run: (input) => {
    const violations: Array<RuleViolation> = [];
    const lines = input.split("\n");

    for (let i = 0; i < lines.length; i++) {
      if (lines[i].length > 0 && lines[i] !== lines[i].trimEnd()) {
        violations.push({
          message: "Line has trailing whitespace",
          line: i + 1,
          snippet: lines[i],
        });
      }
    }

    return violations;
  },
  fix: (input) => {
    return input
      .split("\n")
      .map((line) => line.replace(/[ \t]+$/, ""))
      .join("\n");
  },
};
