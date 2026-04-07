// Core classes
export { Registry } from "@/registry.js";
export { LlmRegistry } from "@/llm-registry.js";

// Default registries with all built-in rules
export { defaultRegistry, defaultLlmRegistry } from "@/defaults.js";

// Config
export { loadConfig, isRuleEnabled } from "@/config.js";
export type { Config, RulesConfig } from "@/config.js";

// Diagnostics
export { fromViolation } from "@/diagnostic.js";
export type { RuleViolation, LintDiagnostic } from "@/diagnostic.js";

// LLM client
export { AnthropicClient, LlmError } from "@/llm-client.js";
export type { LlmAnalyzer, LlmFixViolation } from "@/llm-client.js";

// Rule types
export type { Rule } from "@/rules/index.js";
export type {
  LlmRule,
  LlmResponse,
  LlmViolation,
} from "@/rules/llm-rules/index.js";

// Deterministic rules
export { boldItalicsRule } from "@/rules/bold-italics.js";
export { consecutiveBlankLinesRule } from "@/rules/consecutive-blank-lines.js";
export { descriptionActionRule } from "@/rules/description-action.js";
export { frontmatterRule } from "@/rules/frontmatter.js";
export { frontmatterNameFormatRule } from "@/rules/frontmatter-name-format.js";
export { lineCountRule } from "@/rules/line-count.js";
export { markdownLinksRule } from "@/rules/markdown-links.js";
export { redundantTitleRule } from "@/rules/redundant-title.js";
export { requiredTagsRule } from "@/rules/required-tags.js";
export { trailingWhitespaceRule } from "@/rules/trailing-whitespace.js";
export { unclosedTagsRule } from "@/rules/unclosed-tags.js";
export { whenToUseRule } from "@/rules/when-to-use.js";

// LLM rules
export { cliCommandIndexRule } from "@/rules/llm-rules/cli-command-index.js";
export { duplicateSectionRule } from "@/rules/llm-rules/duplicate-section.js";
export { firstPersonRule } from "@/rules/llm-rules/first-person.js";
export { linkableContentRule } from "@/rules/llm-rules/linkable-content.js";
export { negativeWithoutPositiveRule } from "@/rules/llm-rules/negative-without-positive.js";
export { obviousInstructionsRule } from "@/rules/llm-rules/obvious-instructions.js";
export { processNotIntegrationRule } from "@/rules/llm-rules/process-not-integration.js";
export { redundantExplanationRule } from "@/rules/llm-rules/redundant-explanation.js";
export { skillExampleXmlTagsRule } from "@/rules/llm-rules/skill-example-xml-tags.js";
export { unexplainedUrlRule } from "@/rules/llm-rules/unexplained-url.js";

// Formatter utilities
export {
  formatDiagnostics,
  formatSummary,
  formatListRule,
  formatFixed,
  formatUnfixable,
  formatNote,
  formatWarning,
  formatError,
} from "@/formatter.js";
export type { ListRule } from "@/formatter.js";
