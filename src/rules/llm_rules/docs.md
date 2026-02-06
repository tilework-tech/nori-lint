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

- **`mod.rs`** -- Defines the `LlmRule` trait with four methods: `name() -> &str`, `description() -> &str`, `system_prompt() -> &str`, and `evaluate(input: &str, llm_response: &str) -> Option<RuleViolation>`. Also provides `strip_markdown_fences()`, a shared utility that strips markdown code fence wrappers (`` ``` `` or `` ```json ``) from LLM response strings before JSON parsing. All rule `evaluate()` implementations call this before `serde_json::from_str`.
- **Rule pattern** -- Each LLM rule follows a consistent structure: a `SYSTEM_PROMPT` const with examples and a JSON response schema, serde `Deserialize` structs for the expected response shape (`LlmResponse` + a domain-specific items array), and an `evaluate()` method that strips markdown fences, deserializes the JSON, checks `has_violations` plus a non-empty items array, and constructs a `RuleViolation` with the offending text quoted in the message. Malformed LLM responses return `None` (pass) via `.ok()?`.
- **`redundant_explanation.rs`** -- `RedundantExplanationRule` flags passages that waste tokens explaining concepts an LLM already knows (e.g., "GCP stands for Google Cloud Platform"). JSON response uses `explanations` array of `{text, reason}` objects.
- **`cli_command_index.rs`** -- `CliCommandIndexRule` flags sections that contain CLI command indexes or reference lists (tabular command/description listings, bare command lists, markdown tables of commands). JSON response uses `indexes` array of `{text, reason}` objects. The system prompt explicitly excludes single CLI examples in context or step-by-step workflows.

### Things to Know

- The `LlmRule` trait is synchronous -- the async boundary lives in the `LlmAnalyzer` trait, not in the rule itself. This means rules can be unit-tested with plain strings, no async runtime needed.
- `evaluate()` receives both the original file content and the LLM response. Current rules only use the LLM response, but the `input` parameter is available for rules that need to cross-reference.
- Each rule's system prompt explicitly tells the LLM what NOT to flag to reduce false positives (e.g., project-specific terms for `redundant_explanation`, single contextual CLI examples for `cli_command_index`).
- All rules share the same guard pattern: `if !parsed.has_violations || parsed.<items>.is_empty() { return None; }` -- this means a contradictory LLM response (violations=true but empty array) is treated as a pass.

Created and maintained by Nori.
