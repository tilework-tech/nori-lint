import { LlmRegistry } from "@/llm-registry.js";
import { Registry } from "@/registry.js";
import { boldItalicsRule } from "@/rules/bold-italics.js";
import { consecutiveBlankLinesRule } from "@/rules/consecutive-blank-lines.js";
import { descriptionActionRule } from "@/rules/description-action.js";
import { frontmatterNameFormatRule } from "@/rules/frontmatter-name-format.js";
import { frontmatterRule } from "@/rules/frontmatter.js";
import { lineCountRule } from "@/rules/line-count.js";
import { cliCommandIndexRule } from "@/rules/llm-rules/cli-command-index.js";
import { duplicateSectionRule } from "@/rules/llm-rules/duplicate-section.js";
import { firstPersonRule } from "@/rules/llm-rules/first-person.js";
import { linkableContentRule } from "@/rules/llm-rules/linkable-content.js";
import { negativeWithoutPositiveRule } from "@/rules/llm-rules/negative-without-positive.js";
import { obviousInstructionsRule } from "@/rules/llm-rules/obvious-instructions.js";
import { processNotIntegrationRule } from "@/rules/llm-rules/process-not-integration.js";
import { redundantExplanationRule } from "@/rules/llm-rules/redundant-explanation.js";
import { skillExampleXmlTagsRule } from "@/rules/llm-rules/skill-example-xml-tags.js";
import { unexplainedUrlRule } from "@/rules/llm-rules/unexplained-url.js";
import { markdownLinksRule } from "@/rules/markdown-links.js";
import { redundantTitleRule } from "@/rules/redundant-title.js";
import { requiredTagsRule } from "@/rules/required-tags.js";
import { trailingWhitespaceRule } from "@/rules/trailing-whitespace.js";
import { unclosedTagsRule } from "@/rules/unclosed-tags.js";
import { whenToUseRule } from "@/rules/when-to-use.js";

export function defaultRegistry(): Registry {
  const registry = new Registry();
  registry.register(boldItalicsRule);
  registry.register(consecutiveBlankLinesRule);
  registry.register(descriptionActionRule);
  registry.register(frontmatterRule);
  registry.register(frontmatterNameFormatRule);
  registry.register(lineCountRule);
  registry.register(markdownLinksRule);
  registry.register(redundantTitleRule);
  registry.register(requiredTagsRule);
  registry.register(trailingWhitespaceRule);
  registry.register(unclosedTagsRule);
  registry.register(whenToUseRule);
  return registry;
}

export function defaultLlmRegistry(): LlmRegistry {
  const registry = new LlmRegistry();
  registry.register(cliCommandIndexRule);
  registry.register(duplicateSectionRule);
  registry.register(firstPersonRule);
  registry.register(negativeWithoutPositiveRule);
  registry.register(obviousInstructionsRule);
  registry.register(processNotIntegrationRule);
  registry.register(redundantExplanationRule);
  registry.register(skillExampleXmlTagsRule);
  registry.register(linkableContentRule);
  registry.register(unexplainedUrlRule);
  return registry;
}
