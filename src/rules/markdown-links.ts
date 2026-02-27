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
 * @param trimmed
 */
function isExampleTagOpen(trimmed: string): boolean {
  return (
    trimmed === "<good-example>" ||
    trimmed === "<bad-example>" ||
    trimmed === "<good_example>" ||
    trimmed === "<bad_example>"
  );
}

/**
 *
 * @param trimmed
 */
function isExampleTagClose(trimmed: string): boolean {
  return (
    trimmed === "</good-example>" ||
    trimmed === "</bad-example>" ||
    trimmed === "</good_example>" ||
    trimmed === "</bad_example>"
  );
}

/**
 *
 * @param checkLine
 * @param originalLine
 * @param lineNum
 * @param violations
 */
function findMarkdownLinks(
  checkLine: string,
  originalLine: string,
  lineNum: number,
  violations: Array<RuleViolation>,
): void {
  let i = 0;
  while (i < checkLine.length) {
    if (checkLine[i] === "[") {
      if (i > 0 && checkLine[i - 1] === "!") {
        i++;
        continue;
      }
      const closeBracket = checkLine.indexOf("]", i + 1);
      if (closeBracket === -1) {
        i++;
        continue;
      }
      const linkText = checkLine.slice(i + 1, closeBracket);
      if (linkText.length === 0) {
        i = closeBracket + 1;
        continue;
      }
      if (
        closeBracket + 1 < checkLine.length &&
        checkLine[closeBracket + 1] === "("
      ) {
        const closeParen = checkLine.indexOf(")", closeBracket + 2);
        if (closeParen !== -1) {
          const matched = checkLine.slice(i, closeParen + 1);
          violations.push({
            message: `Markdown link syntax found: ${matched}. Use bare URLs instead`,
            line: lineNum,
            snippet: originalLine,
          });
          i = closeParen + 1;
          continue;
        }
      }
      i = closeBracket + 1;
    } else {
      i++;
    }
  }
}

/**
 *
 * @param line
 */
function replaceMarkdownLinks(line: string): string {
  let result = "";
  let i = 0;

  while (i < line.length) {
    // Preserve inline code spans
    if (line[i] === "`") {
      const codeStart = i;
      i++;
      while (i < line.length && line[i] !== "`") {
        i++;
      }
      if (i < line.length) {
        result += line.slice(codeStart, i + 1);
        i++;
      } else {
        result += line.slice(codeStart);
      }
      continue;
    }

    // Skip image links ![alt](url)
    if (line[i] === "[" && i > 0 && line[i - 1] === "!") {
      result += line[i];
      i++;
      continue;
    }

    if (line[i] === "[") {
      const closeBracket = line.indexOf("]", i + 1);
      if (closeBracket === -1) {
        result += line[i];
        i++;
        continue;
      }
      const linkText = line.slice(i + 1, closeBracket);
      if (linkText.length === 0) {
        result += line.slice(i, closeBracket + 1);
        i = closeBracket + 1;
        continue;
      }
      if (closeBracket + 1 < line.length && line[closeBracket + 1] === "(") {
        const closeParen = line.indexOf(")", closeBracket + 2);
        if (closeParen !== -1) {
          const rawUrl = line.slice(closeBracket + 2, closeParen);
          const spaceIdx = rawUrl.indexOf(" ");
          const url = spaceIdx !== -1 ? rawUrl.slice(0, spaceIdx) : rawUrl;
          result += url;
          i = closeParen + 1;
          continue;
        }
      }
      result += line[i];
      i++;
    } else {
      result += line[i];
      i++;
    }
  }

  return result;
}

export const markdownLinksRule: Rule = {
  name: "markdown_links",
  description:
    "Checks that SKILL.md files do not use markdown link syntax; URLs should be bare links",
  run: (input) => {
    const violations: Array<RuleViolation> = [];
    let inCodeBlock = false;
    let inExampleBlock = false;

    const lines = input.split("\n");
    for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
      const line = lines[lineIdx];
      const lineNum = lineIdx + 1;
      const trimmed = line.trim();

      if (isExampleTagOpen(trimmed)) {
        inExampleBlock = true;
        continue;
      }
      if (isExampleTagClose(trimmed)) {
        inExampleBlock = false;
        continue;
      }
      if (inExampleBlock) continue;

      if (trimmed.startsWith("```")) {
        inCodeBlock = !inCodeBlock;
        continue;
      }
      if (inCodeBlock) continue;

      if (trimmed.startsWith("[")) {
        const bracketEnd = trimmed.indexOf("]");
        if (
          bracketEnd !== -1 &&
          trimmed.slice(bracketEnd + 1).startsWith(": ")
        ) {
          continue;
        }
      }

      const checkLine = stripInlineCode(line);
      findMarkdownLinks(checkLine, line, lineNum, violations);
    }

    return violations;
  },
  fix: (input) => {
    let inCodeBlock = false;
    let inExampleBlock = false;
    const lines = input.split("\n");
    const result: Array<string> = [];

    for (const line of lines) {
      const trimmed = line.trim();

      if (isExampleTagOpen(trimmed)) {
        inExampleBlock = true;
        result.push(line);
        continue;
      }
      if (isExampleTagClose(trimmed)) {
        inExampleBlock = false;
        result.push(line);
        continue;
      }
      if (inExampleBlock) {
        result.push(line);
        continue;
      }

      if (trimmed.startsWith("```")) {
        inCodeBlock = !inCodeBlock;
        result.push(line);
        continue;
      }
      if (inCodeBlock) {
        result.push(line);
        continue;
      }

      if (trimmed.startsWith("[")) {
        const bracketEnd = trimmed.indexOf("]");
        if (
          bracketEnd !== -1 &&
          trimmed.slice(bracketEnd + 1).startsWith(": ")
        ) {
          result.push(line);
          continue;
        }
      }

      result.push(replaceMarkdownLinks(line));
    }

    return result.join("\n");
  },
};
