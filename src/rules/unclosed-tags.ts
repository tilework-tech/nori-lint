import type { Rule } from "@/rules/index.js";

export const unclosedTagsRule: Rule = {
  name: "unclosed_tags",
  description:
    "Checks that all opened XML-style tags have corresponding closing tags",
  run: (input) => {
    const openCounts = new Map<string, number>();
    const closeCounts = new Map<string, number>();

    let rest = input;
    while (true) {
      const start = rest.indexOf("<");
      if (start === -1) break;
      const afterOpen = rest.slice(start + 1);
      const end = afterOpen.indexOf(">");
      if (end === -1) break;

      const tagContent = afterOpen.slice(0, end);
      if (tagContent.startsWith("/")) {
        const name = tagContent.slice(1);
        if (name.length > 0 && !name.includes(" ")) {
          closeCounts.set(name, (closeCounts.get(name) ?? 0) + 1);
        }
      } else if (
        tagContent.length > 0 &&
        !tagContent.includes(" ") &&
        !tagContent.startsWith("!") &&
        !tagContent.endsWith("/")
      ) {
        openCounts.set(tagContent, (openCounts.get(tagContent) ?? 0) + 1);
      }

      rest = afterOpen.slice(end + 1);
    }

    const mismatched: Array<string> = [];

    for (const [tag, open] of openCounts) {
      const close = closeCounts.get(tag) ?? 0;
      if (open !== close) {
        mismatched.push(`<${tag}>: ${open} opening, ${close} closing`);
      }
    }

    for (const tag of closeCounts.keys()) {
      if (!openCounts.has(tag)) {
        mismatched.push(`</${tag}> found without opening <${tag}>`);
      }
    }

    mismatched.sort();

    if (mismatched.length === 0) {
      return [];
    }

    return [
      {
        message: `Unclosed or unmatched tags: ${mismatched.join("; ")}`,
      },
    ];
  },
};
