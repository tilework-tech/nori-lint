# Noridoc: rules

Path: @/src/rules

### Overview

- Contains all lint rule implementations for SKILL.md files, organized into two tiers: deterministic rules (sync) and LLM rules (async-capable)
- Deterministic rules implement the `Rule` trait defined in `mod.rs`; LLM rules implement the `LlmRule` trait defined in `@/src/rules/llm_rules/mod.rs`

### How it fits into the larger codebase

- Deterministic rules are registered into the `Registry` in `@/src/cli.rs` during `run()` -- adding a rule here without registering it has no effect
- LLM rules are registered into the `LlmRegistry` in `@/src/cli.rs`, gated by the presence of a valid config
- `mod.rs` re-exports each rule's submodule (including `llm_rules`) and defines the `Rule` trait
- Both rule types produce `RuleViolation` structs (from `@/src/diagnostic.rs`) which the CLI converts into `LintDiagnostic` records for output
- Integration tests in `@/tests/cli.rs` exercise deterministic rules through the compiled binary; LLM rule unit tests live in `@/src/rules/llm_rules/` alongside the implementation

### Core Implementation

- **`mod.rs`** -- Defines the `Rule` trait with three required methods: `name() -> &str`, `description() -> &str`, `run(&str) -> Option<RuleViolation>`. Re-exports submodules for all deterministic rules and the `llm_rules` submodule.
- **Deterministic rules** -- `line_count.rs`, `redundant_title.rs`, `required_tags.rs`, `unclosed_tags.rs`. These are stateless unit structs implementing `Rule`. They receive the full file content as `&str` and return `Option<RuleViolation>`.
- **`llm_rules/`** -- Submodule containing the `LlmRule` trait and LLM-powered rule implementations. See `@/src/rules/llm_rules/docs.md`.

### Things to Know

- The `Rule` trait (deterministic) and `LlmRule` trait (LLM) are completely separate traits with different method signatures. `Rule::run()` receives only file content. `LlmRule::evaluate()` receives both file content and the LLM's response text. `LlmRule::system_prompt()` provides the prompt sent to the LLM.
- All deterministic rules are stateless. Most are file-level (both `line` and `snippet` are `None` in violations), but `redundant_title` provides line-specific violations with the heading's line number and text snippet.
- Each rule module includes its own `#[cfg(test)]` unit tests verifying pass/fail behavior and boundary conditions.

Created and maintained by Nori.
