# Noridoc: llm_rules

Path: @/src/rules/llm_rules

### Overview

- Defines the `LlmRule` trait and contains rule implementations that use LLM analysis to lint SKILL.md files
- Separates prompt generation (`system_prompt()`) from response evaluation (`evaluate()`), allowing rules to be tested without calling an LLM

### How it fits into the larger codebase

- `LlmRule` is a distinct trait from the deterministic `Rule` trait in `@/src/rules/mod.rs` -- the two rule types have different registries, different execution paths, and different runtime requirements
- LLM rules are registered into `LlmRegistry` (from `@/src/llm_registry.rs`) in `@/src/cli.rs`, only when a valid config with an API key is present
- Execution flows through `run_llm_rules()` in `@/src/cli.rs`, which calls the `LlmAnalyzer` trait (from `@/src/llm_client.rs`) with each rule's `system_prompt()`, then passes the LLM response text to `evaluate()`
- Rules produce `RuleViolation` structs (from `@/src/diagnostic.rs`), the same type used by deterministic rules

### Core Implementation

- **`mod.rs`** -- Defines the `LlmRule` trait with four methods: `name() -> &str`, `description() -> &str`, `system_prompt() -> &str`, and `evaluate(input: &str, llm_response: &str) -> Option<RuleViolation>`. The `input` parameter is the raw SKILL.md content; `llm_response` is the text returned by the LLM.
- **`redundant_explanation.rs`** -- `RedundantExplanationRule` detects when a skill file wastes context window tokens explaining concepts an LLM already knows (e.g., "GCP stands for Google Cloud Platform"). The system prompt instructs Claude to return a JSON object with `has_violations` and `explanations` fields. The `evaluate()` method deserializes the LLM's JSON response and returns a `RuleViolation` listing the offending passages. Fires whenever `has_violations` is true and the explanations list is non-empty.
- **`process_not_integration.rs`** -- `ProcessNotIntegrationRule` detects when a skill file is structured as a reference manual (listing CLI commands, API endpoints, tool capabilities) rather than a step-by-step process with clear workflows. The system prompt teaches the LLM to distinguish "integration" style (declarative tone, parameter docs, capability catalogs) from "process" style (numbered checklists, imperative verbs, `<required>` blocks, conditional branching). The `evaluate()` method deserializes a JSON response with `is_integration`, `confidence`, and `evidence` fields. Only flags a violation when confidence is `"high"` and evidence is non-empty, to avoid false positives on hybrid skills.

### Things to Know

- The `LlmRule` trait is synchronous -- the async boundary lives in the `LlmAnalyzer` trait, not in the rule itself. This means rules can be unit-tested with plain strings, no async runtime needed.
- `evaluate()` receives both the original file content and the LLM response. Current rules only use the LLM response, but the `input` parameter is available for rules that need to cross-reference.
- Each rule defines its own JSON response schema in the system prompt. There is no shared schema across rules.
- Both rules use a `strip_markdown_fences()` helper to handle cases where the LLM wraps its JSON response in markdown code fences. This function is duplicated as a private helper in each rule module.
- Malformed LLM responses are handled gracefully in both rules via `.ok()?` on `serde_json::from_str()` -- they produce `None` (pass) rather than a panic.
- Rules differ in their false-positive gating strategy: `redundant_explanation` fires whenever the LLM says there are violations; `process_not_integration` additionally requires `"high"` confidence before reporting.

Created and maintained by Nori.
