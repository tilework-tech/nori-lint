import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Find URLs in the provided SKILL.md file that appear without sufficient surrounding context explaining what the linked content is or why it matters. A URL should be accompanied by text that tells the reader what they will find at that URL and why they should care.

Examples of unexplained URLs (BAD):
- "Read https://raw.githubusercontent.com/org/repo/abc123/path/to/file.md" — no description of what the file contains or why to read it
- A bare URL on its own line with no surrounding text
- "See https://example.com/some/deep/path" — the URL path alone is not an explanation

Examples of explained URLs (GOOD — do NOT flag):
- "Read the Rust async book for background on executors: https://rust-lang.github.io/async-book/"
- "The CI configuration is documented at https://docs.github.com/en/actions — specifically the section on workflow triggers"
- URLs inside fenced code blocks (these are code, not documentation references)
- URLs inside command examples like \`curl https://api.example.com/v1/health\`
- URLs that are clearly self-describing from context (e.g., after a sentence that explains what the linked resource is)

For each violation found, report the unexplained URL as "text" and why it lacks context as "reason".`;

export const unexplainedUrlRule: LlmRule = {
  name: "unexplained_url",
  description:
    "Checks that URLs in skill files are accompanied by context explaining what they link to",
  systemPrompt: SYSTEM_PROMPT,
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((u) => `"${u.text}"`);
    return {
      message: `File contains URLs without surrounding context explaining what they link to: ${details.join(", ")}`,
    };
  },
};
