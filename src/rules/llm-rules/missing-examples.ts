import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Evaluate the provided SKILL.md file for sections that describe expected behavior, conventions, or patterns that would benefit from concrete \`<good-example>\` and/or \`<bad-example>\` blocks but currently lack them.

A skill teaches an AI agent how to perform a task. Examples are the most effective way to communicate expectations — they show rather than tell. When a skill describes what "good" or "bad" looks like in prose, it should demonstrate with concrete examples.

Sections that SHOULD have examples (flag these if examples are missing):
- Behavioral expectations: "Steps should be actionable and concrete" — what does "actionable" look like?
- Format/convention rules: "Use kebab-case naming" — show good vs bad names
- Style guidance: "Write concise commit messages" — show a good vs bad message
- Quality criteria: "Tests should verify behavior not implementation" — show the difference
- Any instruction where the agent might reasonably interpret it differently without a concrete illustration

Sections that do NOT need examples (do NOT flag these):
- Procedural steps with specific commands: "Run \`npm test\`", "Read the file at path X"
- Steps that reference other skills: "Follow the TDD skill"
- Simple boolean checks: "Verify tests pass"
- Content that already has \`<good-example>\` or \`<bad-example>\` blocks nearby
- TodoWrite instructions and checklist items
- Frontmatter (name, description, etc.)

IMPORTANT: Only flag passages where adding examples would SUBSTANTIALLY improve clarity. Do not flag every minor instruction. Focus on the most impactful gaps.

For each violation found, report the passage that needs examples as "text" and explain why examples would help as "reason".`;

export const missingExamplesRule: LlmRule = {
  name: "missing_examples",
  description:
    "Checks that skill files use example blocks to illustrate behavioral expectations",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((v) => `"${v.text}"`);
    return {
      message: `File contains behavioral guidance that would benefit from <good-example> or <bad-example> blocks: ${details.join(", ")}`,
    };
  },
};
