# Noridoc: rules

Path: @/src/rules

### Overview

- Contains all lint rule implementations for SKILL.md files, organized into two tiers: deterministic rules (sync) and LLM rules (async-capable)
- Deterministic rules conform to the `Rule` type defined in `index.ts`; LLM rules conform to the `LlmRule` type defined in `@/src/rules/llm-rules/index.ts`

### How it fits into the larger codebase

- Deterministic rules are registered into the `Registry` in `@/src/cli.ts` via `defaultRegistry()` -- adding a rule here without registering it has no effect
- LLM rules are registered into the `LlmRegistry` in `@/src/cli.ts` via `defaultLlmRegistry()`, gated by the presence of a valid config
- `index.ts` defines the `Rule` type with `name`, `description`, and `run(input) -> Array<RuleViolation>`
- Both rule types produce `RuleViolation` structs (from `@/src/diagnostic.ts`) which the CLI converts into `LintDiagnostic` records for output
- Unit tests live alongside each rule as `.test.ts` files

### Core Implementation

- **`index.ts`** -- Defines the `Rule` type: an object with `name: string`, `description: string`, and `run: (input: string) => Array<RuleViolation>`.
- **Deterministic rules** -- Exported as constant objects conforming to `Rule`. Each receives full file content as a string and returns an array of violations. Rules include bold/italic detection, line count enforcement, markdown link detection, redundant title detection, required tag checking, and unclosed tag checking.
- **`llm-rules/`** -- Submodule containing the `LlmRule` type and LLM-powered rule implementations. See `@/src/rules/llm-rules/docs.md`.

### Things to Know

- The `Rule` type (deterministic) and `LlmRule` type (LLM) are separate types with different shapes. `Rule.run()` receives only file content. `LlmRule.evaluate()` receives both file content and the LLM's parsed violations. `LlmRule.systemPrompt` provides the prompt sent to the LLM.
- `Rule.run()` returns `Array<RuleViolation>`, allowing a single rule to report multiple violations per file. Line-level rules (bold_italics, markdown_links) may return many violations; file-level rules (line_count, required_tags) return zero or one.
- The `markdown_links` rule tracks `inExampleBlock` state to skip content inside XML example tags (`<good-example>`, `<bad-example>`, `<good_example>`, `<bad_example>`). This pattern is not used by other deterministic rules.
- Both `bold_italics` and `markdown_links` use independent copies of a `stripInlineCode()` helper to replace inline code spans with spaces before scanning.

Created and maintained by Nori.
