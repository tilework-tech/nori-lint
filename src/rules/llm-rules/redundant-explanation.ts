import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Find passages in the provided file where the author wastes tokens explaining concepts that a modern LLM already knows from its training data.

<bad_example>
"GCP stands for Google Cloud Platform"
"JSON is a lightweight data interchange format"
"REST is a style of web API design"
"Git is a version control system"
</bad_example>

These lines should just be removed.

Sometimes this can be subtle.

<bad_example>
7. Main branches are often protected. This is done to ensure that we do not accidentally deploy things to production that contain bugs. Confirm that you are not on the main branch. If you are, ask me before proceeding. NEVER push to main without permission.
</bad_example>

<good_example>
7. Confirm that you are not on the main branch. If you are, ask me before proceeding. NEVER push to main without permission.
</good_example>

<bad_example>
Sometimes there can be conflicts between the branch you are merging and the main branch. Merge main and resolve conflicts if necessary.
</bad_example>

<good_example>
Sometimes there can be conflicts between the branch you are merging and the main branch. Merge main and resolve conflicts if necessary.
</good_example>

In the above examples, the author explains how main branch protection works and that merge conflicts can exist. These are redundant and should be removed.

For each violation found, report the redundant explanation as "text" and why it is redundant as "reason".`;

export const redundantExplanationRule: LlmRule = {
  name: "redundant_explanation",
  description:
    "Checks that skill files do not waste tokens explaining concepts an LLM already knows",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((e) => `"${e.text}"`);
    return {
      message: `File contains explanations of concepts the LLM already knows: ${details.join(", ")}`,
    };
  },
};
