# Noridoc: nori-lint

Path: @/

### Overview

- TypeScript CLI tool **and library** that lints and auto-fixes SKILL.md files (AI agent skill configuration files)
- Uses a two-tier rule system: synchronous deterministic rules (`Rule` type) and async LLM-based rules (`LlmRule` type) that call the Anthropic Messages API for subjective analysis
- LLM rules execute concurrently via `Promise.all`, reducing wall-clock time to roughly the duration of the slowest single API call
- Three subcommands: `lint` (report violations with `--format text|json`), `fix` (auto-fix violations with `--dry-run` support), and `list` (display all available rules with metadata)

### How it fits into the larger codebase

- Part of the Nori monorepo; sits alongside other Nori projects
- Intended to lint SKILL.md files found throughout the Nori skills system (e.g., files under `~/.claude/skills/`)
- JSON output mode enables machine consumption by other tools in the Nori ecosystem
- No runtime dependencies on other Nori projects -- operates as a standalone CLI or importable library
- The `exports` field in `@/package.json` exposes the library entrypoint at `@/src/index.ts`, so `import { ... } from 'nori-lint'` works for programmatic consumers. The `main` and `types` fields point to the compiled output for CommonJS/types resolution.

### Core Implementation

- **Library entrypoint:** `@/src/index.ts` is a barrel export that re-exports the public API: core classes (`Registry`, `LlmRegistry`), factory functions (`defaultRegistry`, `defaultLlmRegistry` from `@/src/defaults.ts`), config utilities, diagnostic types, LLM client, all individual rule objects, and formatter utilities. Library consumers can construct pre-populated registries via the factory functions, or build custom registries by importing individual rules. The `LlmAnalyzer` interface type is exported so consumers can implement their own LLM backend.
- **Entry point:** `@/src/cli.ts` uses `commander` with a subcommand pattern for CLI parsing, `glob` for file discovery, and native `fetch` for HTTP. Running `nori-lint` with no arguments shows help; `nori-lint lint [path]` reports violations; `nori-lint fix [path]` auto-fixes them; `nori-lint list` displays all registered rules. The `run()` function is the async entry point. It self-invokes when run directly via `import.meta.url` check. All text output formatting is delegated to `@/src/formatter.ts`, which provides ESLint-style colored output with file grouping and summary lines using `picocolors`.
- **Two-tier rule system:** Deterministic rules conform to the `Rule` type (in `@/src/rules/index.ts`) and are registered into `Registry` (in `@/src/registry.ts`). LLM rules conform to the `LlmRule` type (in `@/src/rules/llm-rules/index.ts`) and are registered into `LlmRegistry` (in `@/src/llm-registry.ts`). Both produce `RuleViolation` structs from `@/src/diagnostic.ts`. LLM rules may optionally declare `serverTools` (e.g., Anthropic web search) which changes the API call behavior in `@/src/llm-client.ts`.
- **Deterministic fix support:** The `Rule` type has an optional `fix` method. Rules that implement `fix` can auto-correct violations (e.g., `bold_italics`, `markdown_links`, `redundant_title`). Rules without `fix` (e.g., `description_action`, `frontmatter`, `frontmatter_name_format`, `line_count`, `required_tags`, `unclosed_tags`) are reported as unfixable during `nori-lint fix`.
- **LLM fix support:** When a config is present, the `fix` subcommand collects LLM rule violations, then calls `LlmAnalyzer.fixContent()` with all violations batched into a single LLM API call per file. Protection of code blocks, URLs, and uncited content from unwanted modification is handled entirely via the `fixContent()` system prompt in `@/src/llm-client.ts`, which explicitly forbids the LLM from touching fenced code blocks, URLs, or any lines not cited in a violation. The LLM returns the corrected file content via a `apply_fixes` tool_use response.
- **Config-gated LLM execution:** LLM rules only run when a `.nori-lint.json` with a valid `anthropic_api_key` is present. Config can be provided via `--config <path>` or auto-discovered as `.nori-lint.json` in the working directory. Without config, a note is printed to stderr and only deterministic rules run.
- **Concurrent LLM execution:** `runLlmRules()` in `@/src/cli.ts` fires all enabled LLM rules concurrently using `Promise.all`. Progress is printed to stderr before firing requests.
- **Rule filtering:** The config file supports a `rules` object with mutually exclusive `enabled` (allowlist) and `disabled` (denylist) fields. `isRuleEnabled()` in `@/src/config.ts` provides the filtering logic. Filtering is applied at the execution loop level in `@/src/cli.ts`. Registries always contain all rules regardless of config.
- **LLM client:** `@/src/llm-client.ts` defines the `LlmAnalyzer` type (async interface with `analyze()` and `fixContent()`) and `AnthropicClient` class. Uses native `fetch` with `AbortSignal.timeout`. The Anthropic tool_use API enforces common JSON schemas for both lint analysis and fix responses.
- **Build/publish pipeline:** The `prebuild` npm script (`rm -rf build`) cleans the build directory before every build, preventing stale artifacts. The `prepublishOnly` lifecycle hook runs `npm run build && npm test` automatically before `npm publish`, ensuring the published tarball always contains freshly-built output that passes all tests. The build script itself (`@/scripts/build.sh`) runs `tsc`, then `tsc-alias` for path alias resolution, then `chmod +x` on the CLI entry point.
- **CI pipeline:** `@/.github/workflows/ci.yml` runs linting, testing, and build checks -- see `@/.github/workflows/docs.md`

### Things to Know

- CLI uses a subcommand pattern: `nori-lint lint [path]` reports violations (`--format text|json`, `--config <path>`); `nori-lint fix [path]` auto-fixes violations (`--dry-run`, `--config <path>`); `nori-lint list` shows all registered rules (`--format text|json`). Running `nori-lint` with no arguments shows help and exits 0.
- Config auto-discovery: if no `--config` flag is passed, the CLI checks for `.nori-lint.json` in the current working directory. If not found, only deterministic rules run.
- `.nori-lint.json` is gitignored to prevent committing API keys.
- Rule filtering semantics: `rules.enabled` is an allowlist, `rules.disabled` is a denylist, specifying both is a validation error. When no `rules` field exists, all rules run.
- Unknown rule names in `enabled` or `disabled` produce a warning on stderr (not a hard error).
- Exit codes: 0 = no violations, 1 = violations/errors/invalid format/config error
- LLM errors are printed to stderr and cause exit code 1, but do not prevent deterministic rule results from being output
- Uses ESM modules, TypeScript 5.7, Node 22, vitest for testing. `tsconfig.json` uses `module: "Node16"` and `moduleResolution: "Node16"` to properly support the `exports` field in `package.json`.
- Path aliases (`@/` -> `src/`) are resolved by `tsc-alias` at build time
- Stale build protection: `prebuild` clears `build/` before every build, `prepublishOnly` forces a clean build + test before `npm publish`, and CI builds before testing. The vitest config (`@/vitest.config.ts`) excludes `.worktrees/**` from test discovery to prevent stale or divergent test files in git worktree directories from being picked up during `npm test`. The npm-pack simulation test in `@/tests/cli.test.ts` provides end-to-end coverage of the distribution pipeline by packing, extracting, installing, symlinking, and running the CLI.
- Runtime dependencies: `commander` (CLI parsing), `glob` (file discovery), `yaml` (YAML parsing for frontmatter validation), `picocolors` (ANSI color codes for terminal output, auto-respects `NO_COLOR` env var). All other functionality uses Node built-ins.

```
                         cli.ts
                            |
                        run() async
                            |
               commander.parseAsync()  ── --help (exit 0)
                            |
                   no args? ── outputHelp() (exit 0)
                            |
             +--------------+--------------+
             |              |              |
         lint / fix         |          list
             |              |              |
       setupCommand()       |     defaultRegistry() +
       /     |     \        |     defaultLlmRegistry()
 Registry    |  globSync()  |              |
       resolveConfig()      |       format (text/json)
             |              |              |
       Config (optional)    |         exit(0|1)
        /         \         |
  isRuleEnabled()  \        |
  AnthropicClient   \       |
             |       |      |
       +-----+------+      |
       |             |      |
     lint          fix      |
       |             |      |
  run rules     run + fix   |
  diagnostics   rule.fix()  |
  text/json     fixContent()|
       |        --dry-run   |
  exit(0|1)    exit(0|1)    |
```

Created and maintained by Nori.
