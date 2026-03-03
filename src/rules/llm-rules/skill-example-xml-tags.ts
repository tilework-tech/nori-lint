import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Analyze the provided SKILL.md file for examples of intended or unintended behavior that are NOT wrapped in the proper XML tags.

Behavioral examples in SKILL.md files should be wrapped in one of these tag pairs:
- <good_example>...</good_example> or <good-example>...</good-example> for examples of correct/intended behavior
- <bad_example>...</bad_example> or <bad-example>...</bad-example> for examples of incorrect/unintended behavior

A "behavioral example" is text that demonstrates how the AI assistant should or should not behave — e.g., sample dialogue, example responses, example instructions, or before/after comparisons.

<bad_example>
A skill file that contains:

"For instance, you should respond like this: 'I will fix the bug now.'"

This is a behavioral example shown inline without any wrapping tags.
</bad_example>

<good_example>
A skill file where each behavioral example is individually wrapped in the appropriate XML tag (good_example or bad_example), so the reader can clearly distinguish positive and negative examples. Each example appears inside its own tag pair rather than as bare inline text.
</good_example>

Important rules:
- Do NOT flag files that contain no behavioral examples at all. Not every skill needs examples.
- Do NOT flag examples that ARE already wrapped in <good_example>, <bad_example>, <good-example>, or <bad-example> tags.
- Do NOT flag code snippets inside markdown fenced code blocks (\`\`\`...\`\`\`). Code blocks are code, not behavioral examples.
- Do NOT flag general prose descriptions or explanations — only flag text that is clearly demonstrating example behavior.
- ONLY flag text that is clearly a behavioral example but is missing the wrapping tags.

For each violation found, report the unwrapped example text as "text" and explain which tag it should be wrapped in as "reason".`;

export const skillExampleXmlTagsRule: LlmRule = {
  name: "skill_example_xml_tags",
  description:
    "Checks that behavioral examples in skill files are wrapped in <good_example> or <bad_example> tags",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((v) => `"${v.text}" (${v.reason})`);
    return {
      message: `File contains behavioral examples not wrapped in <good_example> or <bad_example> tags: ${details.join(", ")}`,
    };
  },
};
