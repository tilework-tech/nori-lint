# Noridoc: nori-lint

Path: @/

### Overview

- TypeScript CLI tool that lints SKILL.md files (AI agent skill configuration files)
- Uses a two-tier rule system: synchronous deterministic rules (`Rule` type) and async LLM-based rules (`LlmRule` type) that call the Anthropic Messages API for subjective analysis
- LLM rules execute concurrently via `Promise.all`, reducing wall-clock time to roughly the duration of the slowest single API call
- Supports structured output in text (default) or JSON format via `--format text|json`

### How it fits into the larger codebase

- Part of the Nori monorepo; sits alongside other Nori projects
- Intended to lint SKILL.md files found throughout the Nori skills system (e.g., files under `~/.claude/skills/`)
- JSON output mode enables machine consumption by other tools in the Nori ecosystem
- No runtime dependencies on other Nori projects -- operates as a standalone CLI

### Core Implementation

- **Entry point:** `@/src/cli.ts` uses `commander` with a subcommand pattern for CLI parsing, `glob` for file discovery, and native `fetch` for HTTP. Running `nori-lint` with no arguments shows help; `nori-lint lint [path]` runs the linter. The `run()` function is the async entry point that orchestrates the full lint pipeline. It self-invokes when run directly via `import.meta.url` check.
- **Two-tier rule system:** Deterministic rules conform to the `Rule` type (in `@/src/rules/index.ts`) and are registered into `Registry` (in `@/src/registry.ts`). LLM rules conform to the `LlmRule` type (in `@/src/rules/llm-rules/index.ts`) and are registered into `LlmRegistry` (in `@/src/llm-registry.ts`). Both produce `RuleViolation` structs from `@/src/diagnostic.ts`.
- **Config-gated LLM execution:** LLM rules only run when a `.nori-lint.json` with a valid `anthropic_api_key` is present. Config can be provided via `--config <path>` or auto-discovered as `.nori-lint.json` in the working directory. Without config, a note is printed to stderr and only deterministic rules run.
- **Concurrent LLM execution:** `runLlmRules()` in `@/src/cli.ts` fires all enabled LLM rules concurrently using `Promise.all`. Progress is printed to stderr before firing requests.
- **Rule filtering:** The config file supports a `rules` object with mutually exclusive `enabled` (allowlist) and `disabled` (denylist) fields. `isRuleEnabled()` in `@/src/config.ts` provides the filtering logic. Filtering is applied at the execution loop level in `@/src/cli.ts`. Registries always contain all rules regardless of config.
- **LLM client:** `@/src/llm-client.ts` defines the `LlmAnalyzer` type (async interface) and `AnthropicClient` class. Uses native `fetch` with `AbortSignal.timeout`. The Anthropic tool_use API enforces a common JSON schema for LLM responses.
- **CI pipeline:** `@/.github/workflows/ci.yml` runs linting, testing, and build checks -- see `@/.github/workflows/docs.md`

### Things to Know

- CLI uses a subcommand pattern: `nori-lint lint [path]` runs the linter with `--format text|json` (default: `text`), `--config <path>` (optional), and `--help`/`-h`. Running `nori-lint` with no arguments shows help and exits 0. Invalid format values produce an error on stderr and exit code 1.
- Config auto-discovery: if no `--config` flag is passed, the CLI checks for `.nori-lint.json` in the current working directory. If not found, only deterministic rules run.
- `.nori-lint.json` is gitignored to prevent committing API keys.
- Rule filtering semantics: `rules.enabled` is an allowlist, `rules.disabled` is a denylist, specifying both is a validation error. When no `rules` field exists, all rules run.
- Unknown rule names in `enabled` or `disabled` produce a warning on stderr (not a hard error).
- Exit codes: 0 = no violations, 1 = violations/errors/invalid format/config error
- LLM errors are printed to stderr and cause exit code 1, but do not prevent deterministic rule results from being output
- Uses ESM modules, TypeScript 5.7, Node 22, vitest for testing
- Path aliases (`@/` -> `src/`) are resolved by `tsc-alias` at build time
- Runtime dependencies: `commander` (CLI parsing), `glob` (file discovery). All other functionality uses Node built-ins.

```
                        cli.ts
                           |
                     [Node.js async]
                           |
                       run() async
                           |
              commander.parseAsync()  ── --help (exit 0)
                           |
                  no args? ── outputHelp() (exit 0)
                      /    |
             Registry  resolveConfig()
            /                \
   [Rule, Rule, ...]    Config (optional)
                          |       \
                          |    rules: {enabled|disabled}
                          |         |
                          |    isRuleEnabled() filter
                          |   /
                     AnthropicClient + LlmRegistry
                              |
                 globSync("**/SKILL.md") -> find files
                /                          \
   run deterministic rules          runLlmRules (concurrent)
   (skip if !isRuleEnabled)                |
                                    Promise.all
                                           |
                                  [rule.await, rule.await, ...]
                                    (parallel API calls)
                \                          /
           collect Array<LintDiagnostic>
                           |
                 format output (text/json)
                           |
                    exit(0 or 1)
```

Created and maintained by Nori.
