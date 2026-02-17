import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Find sections in the provided file that contain a CLI command index or reference list. These are sections where the author lists out CLI commands, subcommands, or flags in an index-like format rather than providing meaningful context about when and why to use them.

<bad_example>
foo --bar  Use this to do abc
foo --baz  Use this to do xyz
foo --qux  Use this to do 123
</bad_example>

<bad_example>
Available commands:
foo
foo bar
foo baz
foo qux
</bad_example>

<bad_example>
| Command | Description |
|---------|-------------|
| foo bar | Does thing A |
| foo baz | Does thing B |
</bad_example>

These command indexes waste tokens. An LLM already has knowledge of CLI tools from its training data, and a flat list of commands with brief descriptions provides no useful context about *when* or *why* to use them.

Do NOT flag:
- A single CLI example used to illustrate a specific point
- A command shown as part of a step-by-step workflow (e.g., "Run \`foo --bar\` to enable logging, then check the output")
- Code blocks showing example usage with meaningful surrounding context

For each violation found, report the offending command index text as "text" and explain why it wastes tokens as "reason".`;

export const cliCommandIndexRule: LlmRule = {
  name: "cli_command_index",
  description:
    "Checks that skill files do not contain CLI command indexes or reference lists",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((v) => `"${v.text}"`);
    return {
      message: `File contains CLI command indexes that waste tokens: ${details.join(", ")}`,
    };
  },
};
