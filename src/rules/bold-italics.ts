import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

/**
 *
 * @param line
 */
function stripInlineCode(line: string): string {
  let result = "";
  let i = 0;
  while (i < line.length) {
    if (line[i] === "`") {
      const openIdx = i;
      let foundClose = false;
      i++;
      while (i < line.length) {
        if (line[i] === "`") {
          for (let k = openIdx; k <= i; k++) {
            result += " ";
          }
          foundClose = true;
          i++;
          break;
        }
        i++;
      }
      if (!foundClose) {
        result += "`";
      }
    } else {
      result += line[i];
      i++;
    }
  }
  return result;
}

/**
 *
 * @param line
 * @param lineNum
 * @param violations
 */
function findDoubleStarPatterns(
  line: string,
  lineNum: number,
  violations: Array<RuleViolation>,
): void {
  let searchFrom = 0;
  while (true) {
    const open = line.indexOf("**", searchFrom);
    if (open === -1) break;
    const afterOpen = open + 2;
    if (afterOpen >= line.length) break;
    const close = line.indexOf("**", afterOpen);
    if (close === -1) break;
    const inner = line.slice(afterOpen, close);
    if (inner.length > 0 && ![...inner].every((c) => c === "*" || c === " ")) {
      const matched = line.slice(open, close + 2);
      violations.push({
        message: `Bold/italic formatting found: ${matched}`,
        line: lineNum,
        snippet: line,
      });
      searchFrom = close + 2;
    } else {
      searchFrom = afterOpen;
    }
  }
}

/**
 *
 * @param line
 * @param lineNum
 * @param violations
 */
function findDoubleUnderscorePatterns(
  line: string,
  lineNum: number,
  violations: Array<RuleViolation>,
): void {
  let searchFrom = 0;
  while (true) {
    const open = line.indexOf("__", searchFrom);
    if (open === -1) break;
    const afterOpen = open + 2;
    if (afterOpen >= line.length) break;
    const close = line.indexOf("__", afterOpen);
    if (close === -1) break;
    const inner = line.slice(afterOpen, close);
    if (inner.length > 0) {
      const matched = line.slice(open, close + 2);
      violations.push({
        message: `Bold/italic formatting found: ${matched}`,
        line: lineNum,
        snippet: line,
      });
      searchFrom = close + 2;
    } else {
      searchFrom = afterOpen;
    }
  }
}

/**
 *
 * @param line
 * @param lineNum
 * @param violations
 */
function findSingleStarPatterns(
  line: string,
  lineNum: number,
  violations: Array<RuleViolation>,
): void {
  let searchFrom = 0;
  while (searchFrom < line.length) {
    const open = line.indexOf("*", searchFrom);
    if (open === -1) break;
    if (open + 1 < line.length && line[open + 1] === "*") {
      searchFrom = open + 2;
      continue;
    }
    if (open > 0 && line[open - 1] === "*") {
      searchFrom = open + 1;
      continue;
    }
    const afterOpen = open + 1;
    if (afterOpen >= line.length) break;
    let foundClose = false;
    let closeSearch = afterOpen;
    while (closeSearch < line.length) {
      const close = line.indexOf("*", closeSearch);
      if (close === -1) break;
      const isDoubleAfter = close + 1 < line.length && line[close + 1] === "*";
      const isDoubleBefore = close > 0 && line[close - 1] === "*";
      if (isDoubleAfter || isDoubleBefore) {
        closeSearch = close + 1;
        continue;
      }
      const inner = line.slice(afterOpen, close);
      if (inner.length > 0) {
        const matched = line.slice(open, close + 1);
        violations.push({
          message: `Bold/italic formatting found: ${matched}`,
          line: lineNum,
          snippet: line,
        });
        searchFrom = close + 1;
        foundClose = true;
        break;
      } else {
        closeSearch = close + 1;
      }
    }
    if (!foundClose) {
      searchFrom = afterOpen;
    }
  }
}

export const boldItalicsRule: Rule = {
  name: "bold_italics",
  description:
    "Checks that SKILL.md files do not use bold or italic markdown formatting",
  run: (input) => {
    const violations: Array<RuleViolation> = [];
    let inCodeBlock = false;

    const lines = input.split("\n");
    for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
      const line = lines[lineIdx];
      const lineNum = lineIdx + 1;
      const trimmed = line.trim();

      if (trimmed.startsWith("```")) {
        inCodeBlock = !inCodeBlock;
        continue;
      }
      if (inCodeBlock) continue;

      if (trimmed.startsWith("* ")) continue;

      if (
        trimmed.length > 0 &&
        [...trimmed].every((c) => c === "*" || c === " ") &&
        [...trimmed].filter((c) => c === "*").length >= 3
      ) {
        continue;
      }

      const checkLine = stripInlineCode(line);
      findDoubleStarPatterns(checkLine, lineNum, violations);
      findDoubleUnderscorePatterns(checkLine, lineNum, violations);
      findSingleStarPatterns(checkLine, lineNum, violations);
    }

    return violations;
  },
};
