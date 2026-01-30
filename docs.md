# Noridoc: nori-lint

Path: @/

### Overview

- Rust CLI tool that lints SKILL.md files (AI agent skill configuration files)
- Uses a plugin-based architecture: a `Rule` trait + `Registry` pattern allows new lint rules to be added independently
- Supports structured output in text (default) or JSON format via `--format text|json`
- Hybrid crate (lib.rs + main.rs) enabling both unit tests on library code and integration tests on the binary

### How it fits into the larger codebase

- Part of the Nori monorepo at `@/../nori/`; sits alongside other Nori projects
- Intended to lint SKILL.md files found throughout the Nori skills system (e.g., files under `~/.claude/skills/`)
- JSON output mode enables machine consumption by other tools in the Nori ecosystem
- No runtime dependencies on other Nori projects -- operates as a standalone CLI

### Core Implementation

- **Binary entry point:** `@/src/main.rs` calls `nori_lint::cli::run()` and exits with its return code
- **Library root:** `@/src/lib.rs` exposes four public modules: `cli`, `diagnostic`, `registry`, and `rules`
- **CLI orchestration:** `@/src/cli.rs` parses `--format text|json` from CLI args, creates a `Registry`, registers all default rules, walks the directory tree for `SKILL.md` files using `walkdir`, runs every registered rule against each file, collects `LintDiagnostic` structs, and renders output in the selected format
- **Diagnostic types:** `@/src/diagnostic.rs` defines `RuleViolation` (returned by rules) and `LintDiagnostic` (serializable output record with rule name, file path, optional line/snippet, and message)
- **Plugin system:** `@/src/registry.rs` holds the `Registry` struct -- new rules implement the `Rule` trait and get registered in `cli::run()`
- **Rule trait:** defined in `@/src/rules/mod.rs`, requires `name()`, `description()`, and `run() -> Option<RuleViolation>`
- **Rule implementations:** `@/src/rules/` contains individual lint rule modules

### Things to Know

- CLI accepts `--format text|json`; default is `text`. Invalid values produce an error on stderr and exit code 1
- Text output format: `[rule_name] path/to/SKILL.md: error message`
- JSON output format: a JSON array of `LintDiagnostic` objects, each with `rule`, `file`, `line`, `snippet`, and `message` fields
- Exit codes: 0 = no violations found, 1 = at least one violation or error
- File discovery walks from the current working directory, so the binary must be invoked from the intended root
- Runtime dependencies: `walkdir` (file discovery), `serde` with derive feature (serialization), `serde_json` (JSON output)

```
                         main.rs
                            |
                        cli::run()
                       /    |     \
              Registry  WalkDir(".")  parse --format
             /                |               \
    [Rule, Rule, ...]  find SKILL.md files   Text | Json
             \                /
              --- run rules on each file ---
                            |
                collect Vec<LintDiagnostic>
                            |
                  format output (text/json)
                            |
                   exit(0 or 1)
```

Created and maintained by Nori.
