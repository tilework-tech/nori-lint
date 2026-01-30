# Noridoc: src

Path: @/src

### Overview

- Contains the library and binary source for the `nori-lint` CLI
- Organized into a hybrid crate: `main.rs` (thin binary entry point) and `lib.rs` (library exposing `cli`, `diagnostic`, `registry`, and `rules` modules)

### How it fits into the larger codebase

- `main.rs` is the binary entry point compiled into the `nori-lint` executable; it delegates immediately to `cli::run()` in the library
- Integration tests in `@/tests/cli.rs` exercise the compiled binary as a subprocess
- Unit tests live alongside production code in `registry.rs` and `rules/line_count.rs` via `#[cfg(test)]` modules

### Core Implementation

- **`main.rs`** -- Calls `nori_lint::cli::run()` and passes its return value to `std::process::exit()`
- **`lib.rs`** -- Library root that re-exports `cli`, `diagnostic`, `registry`, and `rules` as public modules
- **`diagnostic.rs`** -- Defines two structs:
  - `RuleViolation`: returned by `Rule::run()`, carries `message`, optional `line` number, and optional `snippet`. Not serializable -- it is an internal type that rules produce.
  - `LintDiagnostic`: the output-facing struct, derives `Serialize`. Contains `rule`, `file`, `line`, `snippet`, and `message`. Constructed from a `RuleViolation` via `LintDiagnostic::from_violation()`, which combines the violation data with the rule name and file path.
- **`cli.rs`** -- Orchestrates the lint pipeline: parses `std::env::args()` for an optional directory path argument (defaults to `"."`) and `--format text|json` via `parse_args`, validates the path is a directory, builds a `Registry`, registers default rules, walks the directory tree for `SKILL.md` files, runs each rule, collects `LintDiagnostic` structs into a `Vec`, then renders output (text lines or a single JSON array) based on the selected format
- **`registry.rs`** -- Defines the `Registry` struct that holds `Vec<Box<dyn Rule>>`. The `Rule` trait is defined in `@/src/rules/mod.rs`.
- **`rules/`** -- Submodule containing individual rule implementations; see `@/src/rules/docs.md`

### Things to Know

- The `Rule` trait's `run` method receives the full file content as `&str` and returns `Option<RuleViolation>` -- `None` means the file passes, `Some` carries structured violation data (message, optional line, optional snippet)
- `RuleViolation` is the boundary between rules and the CLI; `LintDiagnostic` is the boundary between the CLI and output. `LintDiagnostic::from_violation()` bridges the two by adding rule name and file path context.
- `cli::run()` strips the root path prefix from discovered file paths before printing, so output shows relative paths like `subdir/SKILL.md` regardless of whether the root was `"."` or an absolute path argument
- Adding a new rule requires: implementing the `Rule` trait (returning `RuleViolation`), re-exporting the module in `rules/mod.rs`, and registering it in `cli::run()`
- `parse_args` in `cli.rs` defaults to `OutputFormat::Text` and root `"."` when no flags are present; returns `Err` for missing or invalid values
- Invalid directory arguments produce an error message on stderr and exit code 1, before any linting occurs

Created and maintained by Nori.
