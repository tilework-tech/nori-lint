# Noridoc: nori-lint

Path: @/

### Overview

- Rust CLI tool that lints SKILL.md files (AI agent skill configuration files)
- Uses a two-tier rule system: synchronous deterministic rules (`Rule` trait) and async LLM-based rules (`LlmRule` trait) that call the Anthropic Messages API for subjective analysis
- LLM rules execute concurrently via I/O multiplexing on a single-threaded tokio runtime, reducing wall-clock time from sum-of-all-API-calls to roughly the duration of the slowest single call
- Supports structured output in text (default) or JSON format via `--format text|json`
- Uses clap for CLI parsing with `--help`/`-h` support
- Hybrid crate (lib.rs + main.rs) enabling both unit tests on library code and integration tests on the binary

### How it fits into the larger codebase

- Part of the Nori monorepo at `@/../nori/`; sits alongside other Nori projects
- Intended to lint SKILL.md files found throughout the Nori skills system (e.g., files under `~/.claude/skills/`)
- JSON output mode enables machine consumption by other tools in the Nori ecosystem
- No runtime dependencies on other Nori projects -- operates as a standalone CLI

### Core Implementation

- **Binary entry point:** `@/src/main.rs` uses `#[tokio::main(flavor = "current_thread")]` to provide an async runtime, then calls `nori_lint::cli::run().await`
- **Library root:** `@/src/lib.rs` exposes seven public modules: `cli`, `config`, `diagnostic`, `llm_client`, `llm_registry`, `registry`, and `rules`
- **CLI orchestration:** `@/src/cli.rs` uses clap's derive API (`#[derive(Parser)]`) to parse CLI arguments: `--format text|json`, `--config <path>`, `--help`/`-h`, and an optional positional directory path. After parsing, it resolves config, builds both registries, walks the directory tree for SKILL.md files, filters rules by the config's enable/disable settings, runs deterministic rules synchronously, then runs LLM rules concurrently via `run_llm_rules()` (if config is present), and renders output.
- **Help output:** `--help` prints usage information plus a dynamically generated list of all registered lint rules and their descriptions, produced by `build_rules_help()` in `cli.rs` via clap's `after_help` attribute. New rules added to the registry automatically appear in help output.
- **Two-tier rule system:** Deterministic rules implement `Rule` (in `@/src/rules/mod.rs`) and are registered into `Registry` (in `@/src/registry.rs`). LLM rules implement `LlmRule` (in `@/src/rules/llm_rules/mod.rs`) and are registered into `LlmRegistry` (in `@/src/llm_registry.rs`). Both produce `RuleViolation` structs from `@/src/diagnostic.rs`. Deterministic rules return `Vec<RuleViolation>`, allowing a single rule invocation to report multiple violations per file.
- **Config-gated LLM execution:** LLM rules only run when a `.nori-lint.json` with a valid `anthropic_api_key` is present. Config can be provided via `--config <path>` or auto-discovered as `.nori-lint.json` in the working directory. Without config, a note is printed to stderr and only deterministic rules run.
- **Concurrent LLM execution:** `run_llm_rules()` in `@/src/cli.rs` fires all enabled LLM rules concurrently using `futures::future::join_all`. Progress is printed to stderr (`"  checking rule_a, rule_b..."`), then all API calls happen in parallel. Wall-clock time is dominated by the slowest single API call rather than the sum of all calls. This is I/O concurrency on the single-threaded tokio runtime (cooperative multitasking), not multi-threading.
- **Rule filtering:** The config file supports a `rules` object with mutually exclusive `enabled` (allowlist) and `disabled` (denylist) fields. `Config::is_rule_enabled()` in `@/src/config.rs` provides the core filtering logic. Filtering is applied at the execution loop level in `@/src/cli.rs` -- both deterministic and LLM rules check `is_rule_enabled()` before running. Registries always contain all rules regardless of config.
- **LLM client abstraction:** `@/src/llm_client.rs` defines the `LlmAnalyzer` trait (async) and `AnthropicClient` struct. The trait enables dependency injection for testing.
- **Config loading:** `@/src/config.rs` reads and validates JSON config files. The `Config` struct holds `anthropic_api_key` (required) and an optional `rules` field of type `RulesConfig`. Validation rejects configs where `rules.enabled` and `rules.disabled` are both specified.
- **CI pipeline:** `@/.github/workflows/ci.yml` runs formatting, linting, and test checks on every push and pull request -- see `@/.github/workflows/docs.md`

### Things to Know

- CLI accepts `--format text|json` (default: `text`), `--config <path>` (optional), and `--help`/`-h`. Invalid values produce a clap error on stderr and exit code 2.
- Config auto-discovery: if no `--config` flag is passed, the CLI checks for `.nori-lint.json` in the current working directory. If found, it is loaded and validated. If not found, only deterministic rules run and a note is printed to stderr.
- `.nori-lint.json` is gitignored to prevent committing API keys. `.nori-lint.example.json` serves as a template showing the `rules.disabled` field.
- Rule filtering semantics: `rules.enabled` is an allowlist (only listed rules run), `rules.disabled` is a denylist (all rules run except listed ones), specifying both is a validation error. When no `rules` field exists, all rules run. An empty `enabled` list disables all rules; an empty `disabled` list enables all rules.
- Unknown rule names in `enabled` or `disabled` produce a warning on stderr (not a hard error) for forward-compatibility with config files that reference rules from newer versions.
- Exit codes: 0 = no violations, 1 = at least one violation, read error, or LLM error, 2 = CLI parse error (clap)
- LLM errors (HTTP failures, parse errors) are printed to stderr and cause exit code 1, but do not prevent deterministic rule results from being output
- **Concurrency model:** LLM rules execute via I/O multiplexing on tokio's `current_thread` runtime. No task spawning occurs; `join_all` awaits all futures in parallel via cooperative multitasking. The `LlmRule` trait does NOT require `Send + Sync` bounds.
- **Progress output:** `run_llm_rules()` prints `"  checking rule_a, rule_b, rule_c..."` to stderr before firing API requests, so users see activity during the batch execution. This output goes to stderr to avoid interfering with `--format json` stdout.
- Runtime dependencies: `clap` with derive feature (CLI parsing), `walkdir` (file discovery), `serde`/`serde_json` (serialization), `reqwest` (async HTTP for Anthropic API), `tokio` (async runtime, `current_thread` flavor, with `time` feature for testing), `futures` (for `join_all`)
- All pushes and PRs must pass CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`

```
                         main.rs
                            |
                     [tokio async runtime]
                     (current_thread flavor)
                            |
                        cli::run()
                            |
                      Cli::parse()  ── clap handles --help (exit 0)
                       /    |            and parse errors (exit 2)
              Registry  resolve_config()
             /                \
    [Rule, Rule, ...]    Config (optional)
                           |       \
                           |    rules: {enabled|disabled}
                           |         |
                           |    is_rule_enabled() filter
                           |   /
                      AnthropicClient + LlmRegistry
                               |
                  WalkDir(root) -> find SKILL.md files
                 /                          \
    run deterministic rules          run_llm_rules (concurrent)
    (skip if !is_rule_enabled)              |
                                   futures::future::join_all
                                   (I/O multiplexing, not threads)
                                             |
                                   [rule_a.await, rule_b.await, ...]
                                     (parallel API calls)
                 \                          /
            collect Vec<LintDiagnostic>
                            |
                  format output (text/json)
                            |
                     exit(0 or 1)
```

Created and maintained by Nori.
