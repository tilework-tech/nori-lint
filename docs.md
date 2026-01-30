# Noridoc: nori-lint

Path: @/

### Overview

- Rust CLI tool that lints SKILL.md files (AI agent skill configuration files)
- Uses a plugin-based architecture: a `Rule` trait + `Registry` pattern allows new lint rules to be added independently
- Hybrid crate (lib.rs + main.rs) enabling both unit tests on library code and integration tests on the binary

### How it fits into the larger codebase

- Part of the Nori monorepo at `@/../nori/`; sits alongside other Nori projects
- Intended to lint SKILL.md files found throughout the Nori skills system (e.g., files under `~/.claude/skills/`)
- No runtime dependencies on other Nori projects -- operates as a standalone CLI

### Core Implementation

- **Binary entry point:** `@/src/main.rs` calls `nori_lint::cli::run()` and exits with its return code
- **Library root:** `@/src/lib.rs` exposes three public modules: `cli`, `registry`, and `rules`
- **CLI orchestration:** `@/src/cli.rs` creates a `Registry`, registers all default rules, walks a directory tree for `SKILL.md` files using `walkdir`, runs every registered rule against each file, prints violations, and returns exit code 0 (clean) or 1 (violations found)
- **Directory argument:** The CLI accepts an optional positional argument specifying the directory to lint. When omitted, it defaults to the current working directory (`"."`). If the argument is not a valid directory, the CLI prints an error to stderr and exits with code 1.
- **Plugin system:** `@/src/registry.rs` defines the `Rule` trait and `Registry` struct -- new rules implement `Rule` and get registered in `cli::run()`
- **Rule implementations:** `@/src/rules/` contains individual lint rule modules

### Things to Know

- Output format: `[rule_name] path/to/SKILL.md: error message`
- Exit codes: 0 = no violations found, 1 = at least one violation found (or invalid directory argument)
- File discovery walks from either the provided directory argument or the current working directory when no argument is given
- The `walkdir` crate is the only runtime dependency; `assert_cmd`, `predicates`, and `tempfile` are dev-only

```
                         main.rs
                            |
                        cli::run()
                       /    |     \
              Registry   parse   WalkDir(root)
             /           args        \
    [Rule, Rule, ...]    |     find SKILL.md files
             \           |          /
              --- run rules on each file ---
                            |
                   print violations
                            |
                   exit(0 or 1)
```

Created and maintained by Nori.
