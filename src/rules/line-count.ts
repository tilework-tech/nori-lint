import type { Rule } from "@/rules/index.js";

const MAX_LINES = 150;

/**
 *
 * @param input
 */
function countLines(input: string): number {
  if (input.length === 0) return 0;
  let count = 1;
  for (let i = 0; i < input.length; i++) {
    if (input[i] === "\n" && i < input.length - 1) {
      count++;
    }
  }
  return count;
}

export const lineCountRule: Rule = {
  name: "line_count",
  description: "Checks that SKILL.md files do not exceed 150 lines",
  run: (input) => {
    const count = countLines(input);
    if (count > MAX_LINES) {
      return [
        {
          message: `File has ${count} lines, exceeding the limit of ${MAX_LINES}`,
        },
      ];
    }
    return [];
  },
};
