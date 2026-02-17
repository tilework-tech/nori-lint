import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `You are reviewing a SKILL.md file. These files are instructions written by a human to configure an AI assistant's behavior. The audience is the AI assistant itself.

Find passages where the author refers to themselves in the third person as "the user" instead of using first person ("me", "my", "I"). Since the human is writing instructions for the AI, they should address themselves in first person.

<bad_example>
"Ask the user to answer the question"
"Tell the user about the error"
"Wait for the user's response"
"Show the user the results"
"If the user wants to proceed"
</bad_example>

<good_example>
"Ask me to answer the question"
"Tell me about the error"
"Wait for my response"
"Show me the results"
"If I want to proceed"
</good_example>

Only flag cases where "the user" clearly refers to the author of the skill file. Do NOT flag:
- References to end users of a product being built (e.g., "validate the user's input" in a web app context)
- Quoted examples or code snippets
- References to third-party users

For each violation found, report the offending passage as "text" and the corrected first-person version as "reason".`;

export const firstPersonRule: LlmRule = {
  name: "first_person",
  description:
    "Checks that skill files use first person ('me', 'I') instead of 'the user'",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((v) => `"${v.text}" -> "${v.reason}"`);
    return {
      message: `File refers to the skill author as 'the user' instead of first person: ${details.join(", ")}`,
    };
  },
};
