# Noridoc: llm_rules

Path: @/src/rules/llm_rules

### Overview

- Defines the `LlmRule` trait and shared response types (`LlmResponse`, `LlmViolation`) used by all LLM-powered lint rules
- Contains rule implementations that use LLM analysis to lint SKILL.md files
- Separates prompt generation (`system_prompt()`) from response evaluation (`evaluate()`), allowing rules to be tested without calling an LLM

### How it fits into the larger codebase

- `LlmRule` is a distinct trait from the deterministic `Rule` trait in `@/src/rules/mod.rs` -- the two rule types have different registries, different execution paths, and different runtime requirements
- LLM rules are registered into `LlmRegistry` (from `@/src/llm_registry.rs`) in `@/src/cli.rs`, only when a valid config with an API key is present
- Execution flows through `run_llm_rules()` in `@/src/cli.rs`, which calls the `LlmAnalyzer` trait (from `@/src/llm_client.rs`) with each rule's `system_prompt()`, then passes the pre-parsed `LlmViolation` slice to `evaluate()`
- `LlmResponse` and `LlmViolation` are the shared contract between `@/src/llm_client.rs` (which deserializes API responses into these types) and this module (which defines the types and consumes them in `evaluate()`)
- Rules produce `RuleViolation` structs (from `@/src/diagnostic.rs`), the same type used by deterministic rules

### Core Implementation

- **`mod.rs`** -- Defines the `LlmRule` trait with four methods: `name() -> &str`, `description() -> &str`, `system_prompt() -> &str`, and `evaluate(input: &str, violations: &[LlmViolation]) -> Option<RuleViolation>`. Also defines two shared structs:
  - `LlmResponse`: has `has_violations: bool` and `violations: Vec<LlmViolation>`, derives `Deserialize`
  - `LlmViolation`: has `text: String` and `reason: String`, derives `Deserialize`
- **Rule implementations** -- Each rule file (`first_person.rs`, `cli_command_index.rs`, `negative_without_positive.rs`, `redundant_explanation.rs`, `obvious_instructions.rs`, `duplicate_section.rs`, `unexplained_url.rs`, `process_not_integration.rs`) follows a uniform pattern:
  - A `SYSTEM_PROMPT` constant describing what to look for and how to use the `text`/`reason` fields
  - A unit struct implementing `LlmRule`
  - `evaluate()` receives `&[LlmViolation]` (already parsed), formats the violations into a human-readable message, and returns `Option<RuleViolation>`
- **Schema enforcement** -- JSON response parsing is not handled by individual rules. The Anthropic tool_use API enforces the `LlmResponse` schema at the API level in `@/src/llm_client.rs`. Rules receive pre-validated, typed data.

### Things to Know

- The `LlmRule` trait is synchronous -- the async boundary lives in the `LlmAnalyzer` trait, not in the rule itself. This means rules can be unit-tested with plain `LlmViolation` slices, no async runtime needed.
- `evaluate()` receives both the original file content (`input`) and the pre-parsed violations. Most rules only use the violations, but the `input` parameter is available for rules that need to cross-reference the original file.
- The `has_violations` check happens in `@/src/cli.rs` before `evaluate()` is called. The CLI gates on `response.has_violations && !response.violations.is_empty()`, so `evaluate()` is only called when there are actual violations to process. Rules still check `violations.is_empty()` defensively.
- System prompts include explicit carve-outs for content that should NOT be flagged (e.g., absolute prohibitions in `negative_without_positive`, single contextual CLI examples for `cli_command_index`, Quick Reference summaries for `duplicate_section`) to reduce false positives.
- System prompts instruct the LLM on how to use the `text` and `reason` fields semantically -- different rules interpret these fields differently (e.g., `first_person` uses `text` for the offending passage and `reason` for the corrected version).

Created and maintained by Nori.
