# Noridoc: llm-rules

Path: @/src/rules/llm-rules

### Overview

- Defines the `LlmRule` type and shared response types (`LlmResponse`, `LlmViolation`) used by all LLM-powered lint rules
- Contains rule implementations that use LLM analysis to lint SKILL.md files
- Separates prompt generation (`systemPrompt`) from response evaluation (`evaluate()`), allowing rules to be tested without calling an LLM

### How it fits into the larger codebase

- `LlmRule` is a distinct type from the deterministic `Rule` type in `@/src/rules/index.ts` -- the two rule types have different registries, different execution paths, and different runtime requirements
- LLM rules are registered into `LlmRegistry` (from `@/src/llm-registry.ts`) in `@/src/cli.ts` via `defaultLlmRegistry()`, only when a valid config with an API key is present
- Execution flows through `runLlmRules()` in `@/src/cli.ts`, which calls the `LlmAnalyzer` type (from `@/src/llm-client.ts`) with each rule's `systemPrompt`, then passes the pre-parsed `LlmViolation` array to `evaluate()`
- `LlmResponse` and `LlmViolation` are the shared contract between `@/src/llm-client.ts` (which extracts API responses into these types) and this module (which defines the types and consumes them in `evaluate()`)
- Rules produce `RuleViolation` structs (from `@/src/diagnostic.ts`), the same type used by deterministic rules

### Core Implementation

- **`index.ts`** -- Defines the `LlmRule` type with four properties: `name: string`, `description: string`, `systemPrompt: string`, and `evaluate: (input, violations) => RuleViolation | null`. Also defines:
  - `LlmResponse`: `{ has_violations: boolean, violations: Array<LlmViolation> }`
  - `LlmViolation`: `{ text: string, reason: string }`
- **Rule implementations** -- Each rule file exports a constant object conforming to `LlmRule`, following a uniform pattern: a `SYSTEM_PROMPT` constant describing what to look for, and an `evaluate()` function that formats the violations into a human-readable message and returns `RuleViolation | null`.
- **Schema enforcement** -- JSON response parsing is not handled by individual rules. The Anthropic tool_use API enforces the `LlmResponse` schema at the API level in `@/src/llm-client.ts`. Rules receive pre-validated, typed data.

### Things to Know

- The `LlmRule` type is synchronous -- the async boundary lives in the `LlmAnalyzer` type, not in the rule itself. Rules can be unit-tested with plain `LlmViolation` arrays, no async needed.
- `evaluate()` receives both the original file content (`input`) and the pre-parsed violations. Most rules only use the violations, but `input` is available for cross-referencing.
- The `has_violations` check happens in `@/src/cli.ts` before `evaluate()` is called. The CLI gates on `response.has_violations && response.violations.length > 0`.
- System prompts include explicit carve-outs for content that should NOT be flagged to reduce false positives.
- System prompts instruct the LLM on how to use the `text` and `reason` fields semantically -- different rules interpret these fields differently (e.g., `first_person` uses `text` for the offending passage and `reason` for the corrected version).

Created and maintained by Nori.
