import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

const HEADING_PATTERN = /^(#{1,6})\s+when\s+to\s+use\s*$/i;

function headingLevel(line: string): number | null {
  const match = line.match(/^(#{1,6})\s/);
  return match ? match[1].length : null;
}

export const whenToUseRule: Rule = {
  name: "when_to_use",
  description:
    'Checks that SKILL.md files do not contain a "When to Use" section (this belongs in the description field)',
  run: (input) => {
    const violations: Array<RuleViolation> = [];
    const lines = input.split("\n");
    let inCodeBlock = false;

    for (let i = 0; i < lines.length; i++) {
      if (lines[i].trimStart().startsWith("```")) {
        inCodeBlock = !inCodeBlock;
        continue;
      }
      if (inCodeBlock) continue;

      if (HEADING_PATTERN.test(lines[i])) {
        violations.push({
          message:
            '"When to Use" section is redundant — this belongs in the frontmatter description field',
          line: i + 1,
          snippet: lines[i],
        });
      }
    }

    return violations;
  },
  fix: (input) => {
    const lines = input.split("\n");
    const linesToRemove = new Set<number>();
    let inCodeBlock = false;

    for (let i = 0; i < lines.length; i++) {
      if (lines[i].trimStart().startsWith("```")) {
        inCodeBlock = !inCodeBlock;
        continue;
      }
      if (inCodeBlock) continue;

      const match = lines[i].match(HEADING_PATTERN);
      if (!match) continue;

      const level = match[1].length;
      linesToRemove.add(i);

      // Remove all lines until next heading at same or higher level (or EOF)
      for (let j = i + 1; j < lines.length; j++) {
        if (lines[j].trimStart().startsWith("```")) {
          inCodeBlock = !inCodeBlock;
        }
        if (inCodeBlock) {
          linesToRemove.add(j);
          continue;
        }

        const nextLevel = headingLevel(lines[j]);
        if (nextLevel !== null && nextLevel <= level) {
          break;
        }
        linesToRemove.add(j);
      }
    }

    if (linesToRemove.size === 0) return input;

    const kept = lines.filter((_, i) => !linesToRemove.has(i));
    const result = kept.join("\n").replace(/\n{3,}/g, "\n\n");
    return result;
  },
};
