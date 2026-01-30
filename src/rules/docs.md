# Noridoc: rules

Path: @/src/rules

### Overview

- Contains individual lint rule implementations for SKILL.md files
- Each rule is a struct implementing the `Rule` trait defined in `@/src/registry.rs`

### How it fits into the larger codebase

- Rules are registered into the `Registry` in `@/src/cli.rs` during `run()` -- adding a rule here without registering it in `cli::run()` has no effect
- `mod.rs` re-exports each rule's submodule; new rules must be added here to be visible to the rest of the crate
- The `Rule` trait contract: `name()` returns a stable identifier used in output formatting, `description()` provides a human-readable summary, and `run(&str)` receives the full file content and returns `None` (pass) or `Some(message)` (violation)

### Core Implementation

- **`line_count.rs`** -- Implements `LineCountRule`, which checks that a SKILL.md file does not exceed 150 lines. Uses `input.lines().count()` and compares against a `MAX_LINES` constant. Returns a message including both the actual count and the limit when violated.

### Things to Know

- Rules are stateless -- each rule struct is a unit struct with no fields, and `run()` operates purely on the input content
- Each rule module includes its own `#[cfg(test)]` unit tests verifying pass/fail behavior and boundary conditions
- The line count boundary is at 150: files with exactly 150 lines pass, files with 151+ lines fail

Created and maintained by Nori.
