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
- **`redundant_explanation.rs`** -- `RedundantExplanationRule` detects when a skill file wastes context window tokens explaining concepts an LLM already knows (e.g., "GCP stands for Google Cloud Platform"). The system prompt instructs Claude to return a JSON object with `has_violations` and `explanations` fields. The `evaluate()` method deserializes the LLM's JSON response and returns a `RuleViolation` listing the offending passages.
- **`negative_without_positive.rs`** -- `NegativeWithoutPositiveRule` flags instructions that tell the reader what NOT to do without providing a corresponding positive alternative (e.g., "Don't use global variables" without suggesting what to use instead). The system prompt instructs Claude to return a JSON object with `has_violations` and `negatives` fields. The system prompt explicitly carves out absolute prohibitions (safety/policy guardrails like "NEVER push to main") so they are not flagged. The `evaluate()` method deserializes the LLM's JSON response and returns a `RuleViolation` listing the offending passages.
- **`process_not_integration.rs`** -- `ProcessNotIntegrationRule` detects when a skill file is structured as a reference manual (listing CLI commands, API endpoints, tool capabilities) rather than a step-by-step process with clear workflows. The system prompt teaches the LLM to distinguish "integration" style (declarative tone, parameter docs, capability catalogs) from "process" style (numbered checklists, imperative verbs, `<required>` blocks, conditional branching). The `evaluate()` method deserializes a JSON response with `is_integration`, `confidence`, and `evidence` fields. Only flags a violation when confidence is `"high"` and evidence is non-empty, to avoid false positives on hybrid skills.

### Things to Know

- The `LlmRule` trait is synchronous -- the async boundary lives in the `LlmAnalyzer` trait, not in the rule itself. This means rules can be unit-tested with plain strings, no async runtime needed.
- `evaluate()` receives both the original file content and the LLM response. Existing rules only use the LLM response, but the `input` parameter is available for rules that need to cross-reference the original file.
- Most LLM rules follow a shared pattern: the system prompt requests a JSON response with a `has_violations` boolean plus a typed array of findings; `evaluate()` deserializes the response, checks for violations, and builds a `RuleViolation` from the findings. `process_not_integration` uses a different schema (`is_integration`, `confidence`, `evidence`) and additionally requires `"high"` confidence before reporting, to avoid false positives on hybrid skills.
- Malformed LLM responses are handled gracefully in all rules via `.ok()?` -- they produce `None` (pass) rather than a panic.
- A shared `strip_markdown_fences()` helper in `mod.rs` strips markdown code fences from LLM responses before JSON parsing, since LLMs sometimes wrap JSON in triple-backtick blocks.
- System prompts include explicit carve-outs for content that should NOT be flagged (e.g., project-specific terms in `redundant_explanation`, absolute prohibitions in `negative_without_positive`) to reduce false positives.

Created and maintained by Nori.
