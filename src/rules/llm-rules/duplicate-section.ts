import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Find sections in the provided SKILL.md file that cover substantially the same ground. This includes:

1. Sections with identical or near-identical headings at the same level (e.g., two "## Red Flags" sections)
2. Sections whose content heavily overlaps — they make the same points, give the same warnings, or list the same items even if the headings differ (e.g., "## Common Mistakes" and "## Red Flags" both listing the same anti-patterns)

What NOT to flag:
- A "Quick Reference" or "Checklist" section that intentionally summarizes a longer process section — this is a valid pattern as long as it is clearly labeled as a summary/reference
- Sections that address the same topic but from genuinely different angles (e.g., "## When to Use" and "## When NOT to Use")
- Repeated structure across different phases or steps (e.g., each phase having its own "Examples" subsection is fine)

For each violation found, report both duplicate section headings joined by " / " as "text" and explain why they overlap as "reason".`;

export const duplicateSectionRule: LlmRule = {
  name: "duplicate_section",
  description:
    "Checks that skill files do not contain sections that cover substantially the same ground",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((s) => `"${s.text}"`);
    return {
      message: `File contains sections that cover substantially the same ground: ${details.join(", ")}`,
    };
  },
};
