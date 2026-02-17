import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Analyze the provided SKILL.md file for instructions that tell the reader what NOT to do without providing a corresponding example or explanation of what TO do instead.

A "negative without positive" is when the author says "don't do X" or "avoid X" or "never do X" but fails to follow up with what the reader SHOULD do instead.

<bad_example>
"Don't use global variables."

This tells the reader what to avoid but gives no guidance on the alternative.
</bad_example>

<good_example>
"Don't use global variables. Instead, pass dependencies as function parameters or use dependency injection."

This tells the reader what to avoid AND what to do instead.
</good_example>

<bad_example>
"Avoid writing long functions."

No guidance on what to do instead.
</bad_example>

<good_example>
"Avoid writing long functions. Extract logical sections into well-named helper functions that each do one thing."

Pairs the negative with a concrete positive alternative.
</good_example>

Important: Do NOT flag absolute prohibitions that are safety/policy rules with no reasonable alternative to suggest. For example, "NEVER push to main without permission" or "NEVER delete production data" are absolute guardrails, not instructional guidance — they should NOT be flagged.

Only flag instructional or guidance-style negatives where a positive alternative would genuinely help the reader know what to do.

For each violation found, report the negative instruction as "text" and what positive alternative is missing as "reason".`;

export const negativeWithoutPositiveRule: LlmRule = {
  name: "negative_without_positive",
  description:
    "Checks that instructions saying what NOT to do include a corresponding positive alternative",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map(
      (v) => `"${v.text}" (suggestion: ${v.reason})`,
    );
    return {
      message: `File contains negative instructions without positive alternatives: ${details.join(", ")}`,
    };
  },
};
