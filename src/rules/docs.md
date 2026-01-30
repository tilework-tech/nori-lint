# Noridoc: rules

Path: @/src/rules

### Overview

- Contains individual lint rule implementations for SKILL.md files
- Each rule is a struct implementing the `Rule` trait defined in `@/src/rules/mod.rs`

### How it fits into the larger codebase

- Rules are registered into the `Registry` in `@/src/cli.rs` during `run()` -- adding a rule here without registering it in `cli::run()` has no effect
- `mod.rs` re-exports each rule's submodule and defines the `Rule` trait; new rules must be added here to be visible to the rest of the crate
- The `Rule` trait contract: `name()` returns a stable identifier used in output formatting, `description()` provides a human-readable summary, and `run(&str)` receives the full file content and returns `None` (pass) or `Some(RuleViolation)` (violation)
- Rules produce `RuleViolation` structs (from `@/src/diagnostic.rs`) which carry a `message`, optional `line` number, and optional `snippet` -- the CLI converts these into `LintDiagnostic` records for output

### Core Implementation

- **`mod.rs`** -- Defines the `Rule` trait with three required methods: `name() -> &str`, `description() -> &str`, `run(&str) -> Option<RuleViolation>`. Imports `RuleViolation` from `@/src/diagnostic.rs`.
- **`line_count.rs`** -- Implements `LineCountRule`, which checks that a SKILL.md file does not exceed 150 lines. Uses `input.lines().count()` and compares against a `MAX_LINES` constant. Returns a `RuleViolation` with `line: None` and `snippet: None` since this is a file-level rule (not tied to a specific line).

### Things to Know

- Rules are stateless -- each rule struct is a unit struct with no fields, and `run()` operates purely on the input content
- Each rule module includes its own `#[cfg(test)]` unit tests verifying pass/fail behavior and boundary conditions
- The line count boundary is at 150: files with exactly 150 lines pass, files with 151+ lines fail
- File-level rules (like `line_count`) set `line` and `snippet` to `None` in the `RuleViolation`; line-specific rules would populate these fields

Created and maintained by Nori.
