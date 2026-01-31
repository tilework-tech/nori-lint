# Noridoc: nori-lint

Path: @/

### Overview

- Rust CLI tool that lints SKILL.md files (AI agent skill configuration files)
- Uses a two-tier rule system: synchronous deterministic rules (`Rule` trait) and async LLM-based rules (`LlmRule` trait) that call the Anthropic Messages API for subjective analysis
- Supports structured output in text (default) or JSON format via `--format text|json`
- Hybrid crate (lib.rs + main.rs) enabling both unit tests on library code and integration tests on the binary

### How it fits into the larger codebase

- Part of the Nori monorepo at `@/../nori/`; sits alongside other Nori projects
- Intended to lint SKILL.md files found throughout the Nori skills system (e.g., files under `~/.claude/skills/`)
- JSON output mode enables machine consumption by other tools in the Nori ecosystem
- No runtime dependencies on other Nori projects -- operates as a standalone CLI

### Core Implementation

- **Binary entry point:** `@/src/main.rs` uses `#[tokio::main(flavor = "current_thread")]` to provide an async runtime, then calls `nori_lint::cli::run().await`
- **Library root:** `@/src/lib.rs` exposes seven public modules: `cli`, `config`, `diagnostic`, `llm_client`, `llm_registry`, `registry`, and `rules`
- **CLI orchestration:** `@/src/cli.rs` is the main pipeline. It parses CLI args (`--format`, `--config`, optional directory path), resolves config, builds both registries, walks the directory tree for SKILL.md files, runs deterministic rules synchronously, then runs LLM rules asynchronously (if config is present), and renders output
- **Two-tier rule system:** Deterministic rules implement `Rule` (in `@/src/rules/mod.rs`) and are registered into `Registry` (in `@/src/registry.rs`). LLM rules implement `LlmRule` (in `@/src/rules/llm_rules/mod.rs`) and are registered into `LlmRegistry` (in `@/src/llm_registry.rs`). Both produce `RuleViolation` structs from `@/src/diagnostic.rs`.
- **Config-gated LLM execution:** LLM rules only run when a `config.json` with a valid `anthropic_api_key` is present. Config can be provided via `--config <path>` or auto-discovered as `config.json` in the working directory. Without config, behavior is identical to the deterministic-only mode.
- **LLM client abstraction:** `@/src/llm_client.rs` defines the `LlmAnalyzer` trait (async) and `AnthropicClient` struct. The trait enables dependency injection for testing.
- **Config loading:** `@/src/config.rs` reads and validates JSON config files. The `Config` struct currently holds a single field: `anthropic_api_key`.
- **CI pipeline:** `@/.github/workflows/ci.yml` runs formatting, linting, and test checks on every push and pull request -- see `@/.github/workflows/docs.md`

### Things to Know

- CLI accepts `--format text|json` (default: `text`) and `--config <path>` (optional). Invalid values produce an error on stderr and exit code 1.
- Config auto-discovery: if no `--config` flag is passed, the CLI checks for `config.json` in the current working directory. If found, it is loaded and validated. If not found, only deterministic rules run.
- `config.json` is gitignored to prevent committing API keys. `defaultConfig.example.json` serves as a template.
- Exit codes: 0 = no violations, 1 = at least one violation, read error, or LLM error
- LLM errors (HTTP failures, parse errors) are printed to stderr and cause exit code 1, but do not prevent deterministic rule results from being output
- Runtime dependencies: `walkdir` (file discovery), `serde`/`serde_json` (serialization), `reqwest` (async HTTP for Anthropic API), `tokio` (async runtime, `current_thread` flavor)

```
                         main.rs
                            |
                     [tokio async runtime]
                            |
                        cli::run()
                       /    |     \
              Registry  parse args  resolve_config()
             /           /   |  \         \
    [Rule, Rule, ...]  root format config  -> Config (optional)
                                            |
                                   AnthropicClient + LlmRegistry
                                            |
                               WalkDir(root) -> find SKILL.md files
                              /                          \
                 run deterministic rules          run LLM rules (if config)
                              \                          /
                         collect Vec<LintDiagnostic>
                                    |
                          format output (text/json)
                                    |
                             exit(0 or 1)
```

Created and maintained by Nori.
