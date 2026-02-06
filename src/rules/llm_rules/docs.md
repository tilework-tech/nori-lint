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
- **Shared rule pattern** -- Each rule implementation follows the same structure: a `SYSTEM_PROMPT` const instructing the LLM to return a JSON object with `has_violations: bool` and a rule-specific array of findings, private `serde::Deserialize` structs to parse the response, and an `evaluate()` method that deserializes and converts to a `RuleViolation`. Malformed LLM responses are handled gracefully via `.ok()?` -- they produce `None` (pass) rather than a panic. The shared `strip_markdown_fences()` utility in `mod.rs` unwraps responses wrapped in triple backticks.
- **`redundant_explanation.rs`** -- `RedundantExplanationRule` detects when a skill file wastes context window tokens explaining concepts an LLM already knows (e.g., "GCP stands for Google Cloud Platform"). The JSON response uses an `explanations` array with `text` and `reason` fields.
- **`first_person.rs`** -- `FirstPersonRule` detects when a SKILL.md file refers to the skill author as "the user" (third person) instead of first person ("me", "my", "I"). Since SKILL.md files are instructions from a human to an AI assistant, the human should address themselves in first person. The JSON response uses a `violations` array with `original` and `suggested` fields. The system prompt explicitly excludes references to end users of a product being built, quoted examples, and third-party users to reduce false positives.

### Things to Know

- The `LlmRule` trait is synchronous -- the async boundary lives in the `LlmAnalyzer` trait, not in the rule itself. This means rules can be unit-tested with plain strings, no async runtime needed.
- `evaluate()` receives both the original file content and the LLM response. Both current rules only use the LLM response, but the `input` parameter is available for rules that need to cross-reference.
- Each rule's system prompt explicitly tells the LLM what NOT to flag to reduce false positives -- this is a key part of the rule design.

Created and maintained by Nori.
