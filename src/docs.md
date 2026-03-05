# Noridoc: src

Path: @/src

### Overview

- Contains the TypeScript source for the `nori-lint` CLI
- Organized into modules: `cli.ts` (entry point and orchestration), `formatter.ts` (all CLI text output formatting), `config.ts`, `diagnostic.ts`, `llm-client.ts`, `llm-registry.ts`, `registry.ts`, and `rules/`

### How it fits into the larger codebase

- `cli.ts` is the entry point compiled into the `nori-lint` binary via the `bin` field in `@/package.json`; it self-invokes via a realpath-based entry point guard that resolves symlinks before comparing `process.argv[1]` against `import.meta.url`
- Integration tests in `@/tests/cli.test.ts` exercise `run()` directly by stubbing `process.argv`, `process.stdout.write`, and `process.stderr.write`
- Unit tests live alongside production code as `.test.ts` files (e.g., `config.test.ts`, `registry.test.ts`)

### Core Implementation

- **`cli.ts`** -- Async entry point. Uses `commander` with a subcommand pattern: the root program shows help when invoked with no arguments. The program-level `.version(version)` call registers `--version`/`-V` flags, with the version string read from `package.json` at runtime by walking up from `__dirname`. Three subcommands are registered: `lint` (report violations), `fix` (auto-fix violations), and `list` (display all available rules with metadata). Shared setup logic (config resolution, registry construction, file discovery, rule validation) is extracted into `setupCommand()` to avoid duplication between `lint` and `fix`. `setupCommand()` returns raw error messages without prefixes -- the caller wraps them in `formatError()`. The `lint` subcommand supports `--format text|json` and `--config`; the `fix` subcommand supports `--dry-run` and `--config`. The `list` subcommand bypasses `setupCommand()` entirely -- it calls `defaultRegistry()` and `defaultLlmRegistry()` directly and requires no path argument, config file, or API key. It supports `--format text|json`; JSON format outputs `{ name, description, type, fixable }` objects. All text output formatting is delegated to `@/src/formatter.ts`. During `fix`, deterministic rules with a `fix` method are applied in-process; LLM violations are batched and fixed via a single `llmClient.fixContent()` call per file. Rules without `fix` are reported as unfixable to stderr.
- **`formatter.ts`** -- Pure functions for all CLI text output. `formatDiagnostics()` groups diagnostics by file and renders them in ESLint-style format with colored file headers, `line:0` location indicators, severity labels, and dimmed rule names. `formatSummary()` renders a green checkmark for zero problems or a red "X" with count for violations. `formatListRule()` renders a rule with bold name, colored `[llm]`/`[fixable]` tags, and dimmed description. `formatFixed()`, `formatUnfixable()`, `formatNote()`, `formatWarning()`, and `formatError()` handle single-line status messages with appropriate color coding. Uses `picocolors` for ANSI color codes, which auto-respects `NO_COLOR` env var and non-TTY output.
- **`diagnostic.ts`** -- Defines `RuleViolation` (produced by both `Rule.run()` and `LlmRule.evaluate()`) and `LintDiagnostic` (the output-facing type). `fromViolation()` converts a `RuleViolation` into a `LintDiagnostic` by attaching rule name and file path. `RuleViolation` has optional `line` and `snippet` fields; `LintDiagnostic` normalizes these to `null` when absent.
- **`config.ts`** -- `loadConfig()` reads a JSON file, validates `anthropic_api_key` is present and non-empty, and validates that `enabled` and `disabled` are not both specified. `isRuleEnabled()` implements allowlist/denylist filtering logic.
- **`llm-client.ts`** -- Defines the `LlmAnalyzer` type (async interface with `analyze()` for linting and `fixContent()` for auto-fixing) and `AnthropicClient` class. Both methods POST to the Anthropic Messages API v1 using tool_use. `analyze()` accepts an optional `serverTools` parameter; when server tools are present, they are merged with the `report_lint_violations` tool, `tool_choice` switches from forced (`{ type: "tool", name: TOOL_NAME }`) to `{ type: "auto" }`, and `max_tokens` increases to the fix-level limit (8192 vs 1024). `extractToolInputFromResponse()` accepts an `allowMissing` parameter (defaults to `false`): when `true`, it returns a no-violations fallback instead of throwing if no `tool_use` block is found in the response (handles cases where the model calls server tools but not the report tool). `fixContent()` uses the `apply_fixes` tool with forced tool_choice and checks for `stop_reason === "max_tokens"` to detect truncated responses. The `fixContent()` system prompt is the primary mechanism for protecting file content from unwanted LLM modifications -- it explicitly forbids modifying fenced code blocks, URLs, lines/sections not cited in a violation, and limits changes to only the specific text cited in each violation. `extractFixFromResponse()` handles fix response parsing. Also exports `LlmFixViolation` type used to pass violation info into `fixContent()`.
- **`llm-registry.ts`** / **`registry.ts`** -- Simple container classes that hold arrays of `LlmRule` or `Rule` objects with `register()` methods and `rules` getters.
- **`rules/`** -- Submodule containing deterministic and LLM rule implementations; see `@/src/rules/docs.md`

### Things to Know

- `run()` returns a numeric exit code rather than calling `process.exit()` directly, enabling testability. The entry point guard at the bottom uses `fs.realpathSync()` to resolve both `process.argv[1]` and the module's own filename (via `fileURLToPath(import.meta.url)`) before comparing them, so that the CLI works correctly when invoked through npm-created symlinks.
- `resolveConfig()` in `cli.ts` implements a two-step lookup: explicit `--config` path first, then `.nori-lint.json` in CWD. If neither exists, `null` is returned and LLM rules are skipped.
- Rule filtering happens at the execution loop in `cli.ts`, not inside the registries. Both `Registry` and `LlmRegistry` always hold all registered rules.
- The unknown rule name warning in `cli.ts` dynamically builds the known-rules list from both registries.
- `RuleViolation` is the shared boundary type between both rule tiers and the CLI output pipeline.
- **Concurrency model** -- LLM rules execute concurrently via `Promise.all` on the Node.js event loop. Each rule's API call is wrapped in a try/catch to prevent one failure from aborting others. Results are processed sequentially after all promises settle.
- `commander` is configured with `exitOverride()` so parse errors throw instead of calling `process.exit()`, allowing the `run()` function to return an exit code. The catch block in `run()` treats `--help`, `-h`, `--version`, and `-V` as success exits (code 0) since Commander throws on these flags when `exitOverride()` is active.

Created and maintained by Nori.
