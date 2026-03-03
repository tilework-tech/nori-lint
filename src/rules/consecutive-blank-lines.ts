import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

export const consecutiveBlankLinesRule: Rule = {
  name: "consecutive_blank_lines",
  description:
    "Checks that SKILL.md files do not contain multiple consecutive blank lines",
  run: (input) => {
    const violations: Array<RuleViolation> = [];
    const lines = input.split("\n");
    let consecutiveBlanks = 0;

    for (let i = 0; i < lines.length; i++) {
      if (lines[i] === "") {
        consecutiveBlanks++;
        if (consecutiveBlanks === 2) {
          violations.push({
            message: "Multiple consecutive blank lines",
            line: i + 1,
          });
        }
      } else {
        consecutiveBlanks = 0;
      }
    }

    return violations;
  },
  fix: (input) => {
    return input.replace(/\n{3,}/g, "\n\n");
  },
};
