import type { Rule } from "@/rules/index.js";

/**
 *
 * @param lines
 */
function skipFrontmatter(lines: Array<string>): number {
  if (lines.length === 0 || lines[0].trim() !== "---") {
    return 0;
  }
  for (let i = 1; i < lines.length; i++) {
    if (lines[i].trim() === "---") {
      return i + 1;
    }
  }
  return lines.length;
}

/**
 *
 * @param line
 */
function isSetextUnderline(line: string): boolean {
  if (line.length === 0) return false;
  return [...line].every((c) => c === "=") || [...line].every((c) => c === "-");
}

export const redundantTitleRule: Rule = {
  name: "redundant_title",
  description: "Checks that SKILL.md files do not start with a title heading",
  run: (input) => {
    const lines = input.split("\n");
    const start = skipFrontmatter(lines);

    let firstContentIdx = -1;
    for (let i = start; i < lines.length; i++) {
      if (lines[i].trim().length > 0) {
        firstContentIdx = i;
        break;
      }
    }

    if (firstContentIdx === -1) return [];

    const line = lines[firstContentIdx];
    const lineNumber = firstContentIdx + 1;

    if (line.startsWith("#")) {
      return [
        {
          message: "File starts with a title heading instead of useful content",
          line: lineNumber,
          snippet: line.trim(),
        },
      ];
    }

    const nextIdx = firstContentIdx + 1;
    if (nextIdx < lines.length) {
      const nextLine = lines[nextIdx].trim();
      if (nextLine.length > 0 && isSetextUnderline(nextLine)) {
        return [
          {
            message:
              "File starts with a title heading instead of useful content",
            line: lineNumber,
            snippet: line.trim(),
          },
        ];
      }
    }

    return [];
  },
};
