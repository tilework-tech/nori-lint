# Noridoc: src

Path: @/src

### Overview

- Contains the TypeScript source for the `nori-lint` CLI
- Organized into modules: `cli.ts` (entry point and orchestration), `config.ts`, `diagnostic.ts`, `llm-client.ts`, `llm-registry.ts`, `registry.ts`, and `rules/`

### How it fits into the larger codebase

- `cli.ts` is the entry point compiled into the `nori-lint` binary via the `bin` field in `@/package.json`; it self-invokes via an `import.meta.url` check
- Integration tests in `@/tests/cli.test.ts` exercise `run()` directly by stubbing `process.argv`, `process.stdout.write`, and `process.stderr.write`
- Unit tests live alongside production code as `.test.ts` files (e.g., `config.test.ts`, `registry.test.ts`)

### Core Implementation

- **`cli.ts`** -- Async entry point. Uses `commander` with a subcommand pattern: the root program shows help when invoked with no arguments. Two subcommands are registered: `lint` (report violations) and `fix` (auto-fix violations). Shared setup logic (config resolution, registry construction, file discovery, rule validation) is extracted into `setupCommand()` to avoid duplication between subcommands. The `lint` subcommand supports `--format text|json` and `--config`; the `fix` subcommand supports `--dry-run` and `--config`. During `fix`, deterministic rules with a `fix` method are applied in-process; LLM violations are batched and fixed via a single `llmClient.fixContent()` call per file. Rules without `fix` are reported as unfixable to stderr.
- **`diagnostic.ts`** -- Defines `RuleViolation` (produced by both `Rule.run()` and `LlmRule.evaluate()`) and `LintDiagnostic` (the output-facing type). `fromViolation()` converts a `RuleViolation` into a `LintDiagnostic` by attaching rule name and file path. `RuleViolation` has optional `line` and `snippet` fields; `LintDiagnostic` normalizes these to `null` when absent.
- **`config.ts`** -- `loadConfig()` reads a JSON file, validates `anthropic_api_key` is present and non-empty, and validates that `enabled` and `disabled` are not both specified. `isRuleEnabled()` implements allowlist/denylist filtering logic.
- **`llm-client.ts`** -- Defines the `LlmAnalyzer` type (async interface with `analyze()` for linting and `fixContent()` for auto-fixing) and `AnthropicClient` class. Both methods POST to the Anthropic Messages API v1 using tool_use with forced `tool_choice`. `analyze()` uses the `report_lint_violations` tool; `fixContent()` uses the `apply_fixes` tool with a higher max_tokens (8192 vs 1024) and checks for `stop_reason === "max_tokens"` to detect truncated responses. The `fixContent()` system prompt is the primary mechanism for protecting file content from unwanted LLM modifications -- it explicitly forbids modifying fenced code blocks, URLs, lines/sections not cited in a violation, and limits changes to only the specific text cited in each violation. `extractToolInputFromResponse()` and `extractFixFromResponse()` handle response parsing for each path respectively. Also exports `LlmFixViolation` type used to pass violation info into `fixContent()`.
- **`llm-registry.ts`** / **`registry.ts`** -- Simple container classes that hold arrays of `LlmRule` or `Rule` objects with `register()` methods and `rules` getters.
- **`rules/`** -- Submodule containing deterministic and LLM rule implementations; see `@/src/rules/docs.md`

### Things to Know

- `run()` returns a numeric exit code rather than calling `process.exit()` directly, enabling testability. The `import.meta.url` guard at the bottom calls `process.exit()` only when run as a script.
- `resolveConfig()` in `cli.ts` implements a two-step lookup: explicit `--config` path first, then `.nori-lint.json` in CWD. If neither exists, `null` is returned and LLM rules are skipped.
- Rule filtering happens at the execution loop in `cli.ts`, not inside the registries. Both `Registry` and `LlmRegistry` always hold all registered rules.
- The unknown rule name warning in `cli.ts` dynamically builds the known-rules list from both registries.
- `RuleViolation` is the shared boundary type between both rule tiers and the CLI output pipeline.
- **Concurrency model** -- LLM rules execute concurrently via `Promise.all` on the Node.js event loop. Each rule's API call is wrapped in a try/catch to prevent one failure from aborting others. Results are processed sequentially after all promises settle.
- `commander` is configured with `exitOverride()` so parse errors throw instead of calling `process.exit()`, allowing the `run()` function to return an exit code.

Created and maintained by Nori.
