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

- **`cli.ts`** -- Async entry point. Uses `commander` for CLI parsing with `exitOverride()` to catch parse errors. Orchestrates: config resolution, registry construction, file discovery via `globSync("**/SKILL.md")`, deterministic rule execution (synchronous per file), LLM rule execution (concurrent via `Promise.all`), and output formatting (text or JSON to stdout).
- **`diagnostic.ts`** -- Defines `RuleViolation` (produced by both `Rule.run()` and `LlmRule.evaluate()`) and `LintDiagnostic` (the output-facing type). `fromViolation()` converts a `RuleViolation` into a `LintDiagnostic` by attaching rule name and file path. `RuleViolation` has optional `line` and `snippet` fields; `LintDiagnostic` normalizes these to `null` when absent.
- **`config.ts`** -- `loadConfig()` reads a JSON file, validates `anthropic_api_key` is present and non-empty, and validates that `enabled` and `disabled` are not both specified. `isRuleEnabled()` implements allowlist/denylist filtering logic.
- **`llm-client.ts`** -- Defines the `LlmAnalyzer` type (async `analyze()` returning `Promise<LlmResponse>`) and `AnthropicClient` class that POSTs to the Anthropic Messages API v1 using tool_use with forced `tool_choice` to enforce a common JSON schema. `extractToolInputFromResponse()` finds the `tool_use` content block and extracts the typed input. Uses native `fetch` with `AbortSignal.timeout(60000)`.
- **`llm-registry.ts`** / **`registry.ts`** -- Simple container classes that hold arrays of `LlmRule` or `Rule` objects with `register()` methods and `rules` getters.
- **`rules/`** -- Submodule containing deterministic and LLM rule implementations; see `@/src/rules/docs.md`

### Things to Know

- `run()` returns a numeric exit code rather than calling `process.exit()` directly, enabling testability. The `import.meta.url` guard at the bottom calls `process.exit()` only when run as a script.
- `resolveConfig()` in `cli.ts` implements a two-step lookup: explicit `--config` path first, then `.nori-lint.json` in CWD. If neither exists, `null` is returned and LLM rules are skipped.
- Rule filtering happens at the execution loop in `cli.ts`, not inside the registries. Both `Registry` and `LlmRegistry` always hold all registered rules.
- The unknown rule name warning in `cli.ts` dynamically builds the known-rules list from both registries, unlike the Rust version which used a hardcoded list for LLM rule names.
- `RuleViolation` is the shared boundary type between both rule tiers and the CLI output pipeline.
- **Concurrency model** -- LLM rules execute concurrently via `Promise.all` on the Node.js event loop. Each rule's API call is wrapped in a try/catch to prevent one failure from aborting others. Results are processed sequentially after all promises settle.
- `commander` is configured with `exitOverride()` so parse errors throw instead of calling `process.exit()`, allowing the `run()` function to return an exit code.

Created and maintained by Nori.
