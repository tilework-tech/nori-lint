import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Determine whether the provided SKILL.md file is an "integration" (a reference manual) or a "process" (a step-by-step workflow).

A **process skill** defines a clear workflow the agent must follow. It has:
- Numbered checklists of sequential steps inside \`<required>\` blocks
- Instructions to add steps to a TodoWrite list
- Imperative verbs as step openers: "Write", "Verify", "Run", "Confirm", "Check", "Push"
- Named phases with explicit ordering ("Phase 1", "Phase 2", or "You MUST complete each phase before proceeding")
- Conditional branching ("If tests fail: ... If tests pass: ...")
- Loop constructs ("Follow these steps in a loop until...")
- Stop conditions and red flags

An **integration skill** is essentially a reference manual documenting what a tool can do. It has:
- CLI command templates with parameter documentation (e.g., \`--name (required): Clear, searchable title\`)
- API endpoint listings or mode/capability catalogs
- Platform comparison tables
- Output format descriptions ("Returns confirmation with artifact ID")
- Declarative tone ("Supports...", "Available...", "Returns...")
- No \`<required>\` block with a numbered checklist

A skill can be a hybrid — some process content mixed with reference material. Only flag a skill as an integration if the **majority** of its content is reference/catalog material rather than process steps.

Only set has_violations to true if you have HIGH confidence that this file is an integration/reference manual rather than a process. If confidence is medium or low, set has_violations to false.

For each piece of evidence, report the passage that indicates integration style as "text" and explain why it is integration-style content as "reason".`;

export const processNotIntegrationRule: LlmRule = {
  name: "process_not_integration",
  description:
    "Checks that skill files define a process rather than just documenting a tool integration",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((e) => `"${e.text}"`);
    return {
      message: `Skill file reads as a tool integration/reference manual rather than a process: ${details.join(", ")}`,
    };
  },
};
