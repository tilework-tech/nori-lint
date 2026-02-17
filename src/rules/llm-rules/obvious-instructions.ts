import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Find instructions in the provided SKILL.md file that are so generic and obvious that any LLM would already follow them without being told. These are instructions that waste tokens by stating things the LLM already knows to do.

<bad_example>
"Read code changes carefully"
"Understand the scope of modifications"
"Keep code modular where possible"
"Follow best practices"
"Ensure code quality"
"Write clean code"
"Maintain readability"
"Use meaningful variable names"
"Add comments where necessary"
"Handle errors appropriately"
"Test your changes"
"Review your work"
"Pay attention to detail"
"Consider edge cases"
"Think about performance"
"Ensure proper documentation"
"Follow coding standards"
"Keep functions small"
"Avoid code duplication"
"Understand the codebase"
</bad_example>

These should be removed because they add no value. An LLM already knows to do all of these things.

Instructions that ARE valuable and should NOT be flagged:

<good_example>
"NEVER push to main without permission"
"Use the tokio runtime with current_thread flavor"
"Always run cargo clippy before committing"
"Prefix all API routes with /v2/"
"Log errors to stderr, not stdout"
</good_example>

These are valuable because they are specific, project-contextual, or override default LLM behavior.

The key distinction: if the instruction could appear in ANY project's guidelines and adds nothing project-specific, it is obvious. If it constrains behavior in a specific, non-default way, it is valuable.

For each violation found, report the obvious instruction as "text" and explain why it is obvious as "reason".`;

export const obviousInstructionsRule: LlmRule = {
  name: "obvious_instructions",
  description:
    "Checks that skill files do not contain generic instructions any LLM would already follow",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((i) => `"${i.text}"`);
    return {
      message: `File contains obvious instructions any LLM would already follow: ${details.join(", ")}`,
    };
  },
};
