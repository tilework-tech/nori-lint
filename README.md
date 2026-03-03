# nori-lint

> This README was AI-generated.

An opinionated linter for `SKILL.md` files — follow best practices, don't configure slop.

## Why opinionated?

SKILL.md files are prompt-engineered instructions consumed by LLMs. Most LLMs will write verbose slop skills that do not work. Every wasted word burns context window space in every agent session, degrading agent ability and costing you money.

nori-lint enforces two principles:

- **Token efficiency:** Don't waste tokens on things the LLM already knows (redundant explanations, obvious instructions, markdown formatting it doesn't render).
- **Structural discipline:** Skills should be concise processes with required checklists, not sprawling reference manuals.

You can disable any rule you disagree with via configuration.

## Ecosystem

nori-lint is part of a broader set of tools for building and managing AI agent configurations:

- [Nori Skillsets](https://noriskillsets.com) — Registry and CLI for downloading, publishing, and switching between packaged sets of AI agent skills. Teams can get private registries, which also support automatic skill generation from coding session transcripts.
- [Nori Sessions](https://norisessions.com) — Managed agent runtime platform for running AI coding agents in cloud environments with security, persistence, and integrations. Like Stripe or Ramp, but off the shelf and personalized to your org.
- [Nori](https://usenori.ai) — Homepage.

If you are struggling to adopt AI or are feeling AI FOMO, get in touch!

## Installation

Requires Node.js 22+.

```bash
npm install
npm run build
npm link
```

This makes the `nori-lint` command available globally.

## Usage

Running `nori-lint` with no arguments shows help.

### Lint

```
nori-lint lint [path] [options]
```

Finds all `SKILL.md` files recursively under `[path]` (defaults to `.`) and reports violations.

| Option | Default | Description |
|---|---|---|
| `--format <format>` | `text` | Output format: `text` or `json` |
| `--config <path>` | — | Path to config file |

Exit code 0 means no violations. Exit code 1 means violations were found.

### Fix

```
nori-lint fix [path] [options]
```

Auto-fixes violations in place. Deterministic rules are fixed directly. LLM rule violations are batched into a single API call per file.

| Option | Default | Description |
|---|---|---|
| `--config <path>` | — | Path to config file |
| `--dry-run` | `false` | Show what would change without writing |

## Rules

nori-lint has a two-tier rule system.

### Static rules

These run without any API key. They are fast, deterministic, and pattern-based.

| Rule | Fixable | Description |
|---|---|---|
| `bold_italics` | Yes | Bold/italic markdown formatting wastes tokens for an LLM reader. |
| `consecutive_blank_lines` | Yes | Multiple consecutive blank lines waste tokens. |
| `description_action` | No | Frontmatter `description` must start with "Use " — skills should say when to use them, not what they do. |
| `frontmatter` | No | YAML frontmatter must have valid structure and required `name`/`description` fields per the agentskills.io spec. |
| `frontmatter_name_format` | No | Frontmatter `name` must be lowercase alphanumeric with hyphens per the agentskills.io naming convention. |
| `line_count` | No | Files must not exceed 150 lines. |
| `markdown_links` | Yes | Use bare URLs instead of `[text](url)` markdown link syntax. |
| `redundant_title` | Yes | Title headings are redundant — the filename and frontmatter already identify the skill. |
| `required_tags` | No | Every skill must contain at least one `<required>` block. |
| `trailing_whitespace` | Yes | Trailing whitespace wastes tokens. |
| `unclosed_tags` | No | All opened XML-style tags must have matching closing tags. |

### LLM rules

These call the Anthropic API for semantic analysis. They only run when a config file with a valid `anthropic_api_key` is present.

| Rule | Description |
|---|---|
| `cli_command_index` | Flags CLI command reference lists — an LLM already knows these from training data. |
| `duplicate_section` | Flags sections that cover substantially the same ground. |
| `first_person` | The skill author should refer to themselves as "I/me/my", not "the user". |
| `negative_without_positive` | "Don't do X" instructions must pair with a concrete "do Y instead". |
| `obvious_instructions` | Flags generic instructions any LLM already follows ("write clean code", "handle errors"). |
| `process_not_integration` | Skills should be step-by-step workflows, not reference manuals. |
| `redundant_explanation` | Flags explanations of concepts an LLM already knows. |
| `skill_example_xml_tags` | Behavioral examples should be wrapped in `<good_example>` or `<bad_example>` tags. |
| `unexplained_url` | URLs must have surrounding context explaining what they link to and why. |

## Configuration

Create a `.nori-lint.json` file in your project root, or pass `--config <path>`.

```json
{
  "anthropic_api_key": "sk-ant-your-api-key-here",
  "rules": {
    "disabled": ["obvious_instructions", "redundant_explanation"]
  }
}
```

- `anthropic_api_key` — Required for LLM rules. Without it, only static rules run.
- `rules.enabled` — Allowlist: only these rules run.
- `rules.disabled` — Denylist: all rules except these run.
- `enabled` and `disabled` are mutually exclusive.

The `.nori-lint.json` file is gitignored by default to avoid committing API keys. See `.nori-lint.example.json` for a template.
