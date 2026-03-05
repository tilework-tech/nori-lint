import type { RuleViolation } from "@/diagnostic.js";
import type { LlmRule, LlmViolation } from "@/rules/llm-rules/index.js";

const SYSTEM_PROMPT = `Find sections or passages in the provided SKILL.md file where the author has inlined reference material, tutorials, API documentation, or tool guides that would be better served by linking to the canonical external documentation URL.

Use web search to find the correct canonical URL before suggesting it. Do not guess URLs.

<bad_example>
"GitHub Actions workflows are defined in YAML files stored in the .github/workflows directory of your repository. Each workflow file defines one or more jobs, and each job runs on a runner. You can configure triggers using the 'on' key, which supports events like push, pull_request, schedule, and workflow_dispatch."
</bad_example>

This restates the GitHub Actions docs. Instead, the skill should link to the official documentation.

<bad_example>
"Commander.js uses .command() to define subcommands, .option() for flags, and .action() for handlers. The .parse() method processes argv."
</bad_example>

This restates the Commander.js API. Link to the README or docs instead.

<good_example>
"Run the linter before committing. We use a strict config that disallows any warnings."
</good_example>

This is a project-specific instruction — no external docs to link to.

<good_example>
"See the Docker documentation for multi-stage builds: https://docs.docker.com/build/building/multi-stage/"
</good_example>

This already links to the docs. Do not flag it.

Do NOT flag:
- Project-specific workflows, conventions, or opinionated instructions
- Content that already contains a URL for further reading
- Brief passing mentions of a tool or concept (one sentence or less)
- Content inside fenced code blocks or example tags
- Instructions on how to use a tool in a specific way for this project (even if the tool has docs)

For each violation found, report the inlined content as "text" and include a suggested canonical URL as "reason" (e.g., "Consider linking to: https://docs.example.com/relevant-page").`;

export const linkableContentRule: LlmRule = {
  name: "linkable_content",
  description:
    "Checks that skill files link to external documentation instead of restating it inline",
  systemPrompt: SYSTEM_PROMPT,
  serverTools: [
    { type: "web_search_20250305", name: "web_search", max_uses: 5 },
  ],
  evaluate: (
    _input: string,
    violations: Array<LlmViolation>,
  ): RuleViolation | null => {
    if (violations.length === 0) return null;
    const details = violations.map((v) => `"${v.text}" — ${v.reason}`);
    return {
      message: `File contains content that could link to external documentation instead of restating it: ${details.join("; ")}`,
    };
  },
};
