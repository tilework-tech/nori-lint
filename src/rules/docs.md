# Noridoc: rules

Path: @/src/rules

### Overview

- Contains individual lint rule implementations for SKILL.md files
- Each rule is a stateless unit struct implementing the `Rule` trait defined in `@/src/rules/mod.rs`
- Rules fall into two categories: structural rules (e.g., line count limits) and content rules (e.g., presence and well-formedness of XML-style tags like `<required>`)

### How it fits into the larger codebase

- Rules are registered into the `Registry` in `@/src/cli.rs` during `run()` -- adding a rule here without registering it in `cli::run()` has no effect
- `mod.rs` re-exports each rule's submodule and defines the `Rule` trait; new rules must be added here to be visible to the rest of the crate
- The `Rule` trait contract: `name()` returns a stable identifier used in output formatting, `description()` provides a human-readable summary, and `run(&str)` receives the full file content and returns `None` (pass) or `Some(RuleViolation)` (violation)
- Rules produce `RuleViolation` structs (from `@/src/diagnostic.rs`) which carry a `message`, optional `line` number, and optional `snippet` -- the CLI converts these into `LintDiagnostic` records for output
- Integration tests in `@/tests/cli.rs` exercise each rule through the compiled binary; unit tests live alongside each rule implementation in `#[cfg(test)]` modules

### Core Implementation

- **`mod.rs`** -- Defines the `Rule` trait with three required methods: `name() -> &str`, `description() -> &str`, `run(&str) -> Option<RuleViolation>`. Imports `RuleViolation` from `@/src/diagnostic.rs`.
- **`line_count.rs`** -- `LineCountRule` checks that a SKILL.md file does not exceed 150 lines. Uses `input.lines().count()` and compares against a `MAX_LINES` constant. File-level rule (`line: None`, `snippet: None`).
- **`required_tags.rs`** -- `RequiredTagsRule` checks that a SKILL.md file contains at least one `<required>...</required>` tag pair. Both `<required>` and `</required>` must be present in the content for the file to pass. This enforces a core Nori convention where `<required>` blocks contain critical checklist steps that agents must follow. File-level rule.
- **`unclosed_tags.rs`** -- `UnclosedTagsRule` checks that all opened XML-style tags have matching closing tags. Scans the file content character by character for `<...>` sequences, builds a `HashMap<&str, usize>` of open and close counts per tag name, then reports any mismatches. Tags containing spaces (HTML attributes) and tags starting with `!` (comments like `<!-- -->`) are skipped. Reports both unclosed openers and orphan closers. File-level rule.

### Things to Know

- All rules are stateless -- each is a unit struct with no fields, and `run()` operates purely on the input content string
- All current rules are file-level (both `line` and `snippet` are `None` in violations); the `RuleViolation` struct supports line-specific rules but none exist yet
- Each rule module includes its own `#[cfg(test)]` unit tests verifying pass/fail behavior and boundary conditions
- The `unclosed_tags` rule counts opening and closing tags independently per tag name -- it detects both `<tag>` without `</tag>` and `</tag>` without `<tag>`, including cases where counts are unequal (e.g., two openers but only one closer)
- The `required_tags` rule requires **both** `<required>` and `</required>` to be present -- a file with only one of them still fails

Created and maintained by Nori.
