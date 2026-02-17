#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

import { Command } from "commander";
import { globSync } from "glob";

import { loadConfig, isRuleEnabled } from "@/config.js";
import { fromViolation } from "@/diagnostic.js";
import { AnthropicClient } from "@/llm-client.js";
import { LlmRegistry } from "@/llm-registry.js";
import { Registry } from "@/registry.js";
import { boldItalicsRule } from "@/rules/bold-italics.js";
import { lineCountRule } from "@/rules/line-count.js";
import { cliCommandIndexRule } from "@/rules/llm-rules/cli-command-index.js";
import { duplicateSectionRule } from "@/rules/llm-rules/duplicate-section.js";
import { firstPersonRule } from "@/rules/llm-rules/first-person.js";
import { negativeWithoutPositiveRule } from "@/rules/llm-rules/negative-without-positive.js";
import { obviousInstructionsRule } from "@/rules/llm-rules/obvious-instructions.js";
import { processNotIntegrationRule } from "@/rules/llm-rules/process-not-integration.js";
import { redundantExplanationRule } from "@/rules/llm-rules/redundant-explanation.js";
import { unexplainedUrlRule } from "@/rules/llm-rules/unexplained-url.js";
import { markdownLinksRule } from "@/rules/markdown-links.js";
import { redundantTitleRule } from "@/rules/redundant-title.js";
import { requiredTagsRule } from "@/rules/required-tags.js";
import { unclosedTagsRule } from "@/rules/unclosed-tags.js";

import type { Config } from "@/config.js";
import type { LintDiagnostic } from "@/diagnostic.js";
import type { LlmAnalyzer } from "@/llm-client.js";

/**
 *
 */
function defaultRegistry(): Registry {
  const registry = new Registry();
  registry.register(boldItalicsRule);
  registry.register(lineCountRule);
  registry.register(markdownLinksRule);
  registry.register(redundantTitleRule);
  registry.register(requiredTagsRule);
  registry.register(unclosedTagsRule);
  return registry;
}

/**
 *
 */
function defaultLlmRegistry(): LlmRegistry {
  const registry = new LlmRegistry();
  registry.register(cliCommandIndexRule);
  registry.register(duplicateSectionRule);
  registry.register(firstPersonRule);
  registry.register(negativeWithoutPositiveRule);
  registry.register(obviousInstructionsRule);
  registry.register(processNotIntegrationRule);
  registry.register(redundantExplanationRule);
  registry.register(unexplainedUrlRule);
  return registry;
}

/**
 *
 * @param cliConfigPath
 */
function resolveConfig(cliConfigPath: string | undefined): Config | null {
  if (cliConfigPath) {
    return loadConfig(cliConfigPath);
  }

  const cwdConfig = path.join(process.cwd(), ".nori-lint.json");
  if (fs.existsSync(cwdConfig)) {
    return loadConfig(cwdConfig);
  }

  return null;
}

/**
 *
 * @param client
 * @param llmRegistry
 * @param config
 * @param content
 * @param displayStr
 * @param diagnostics
 * @param hasLlmError
 * @param hasLlmError.value
 */
async function runLlmRules(
  client: LlmAnalyzer,
  llmRegistry: LlmRegistry,
  config: Config,
  content: string,
  displayStr: string,
  diagnostics: Array<LintDiagnostic>,
  hasLlmError: { value: boolean },
): Promise<void> {
  const enabledRules = llmRegistry.rules.filter((rule) =>
    isRuleEnabled(config, rule.name),
  );

  if (enabledRules.length === 0) return;

  const ruleNames = enabledRules.map((r) => r.name);
  process.stderr.write(`  checking ${ruleNames.join(", ")}...\n`);

  const results = await Promise.all(
    enabledRules.map(async (rule) => {
      try {
        const response = await client.analyze(rule.systemPrompt, content);
        return { rule, response, error: null };
      } catch (e) {
        return { rule, response: null, error: e };
      }
    }),
  );

  for (const { rule, response, error } of results) {
    if (error) {
      process.stderr.write(
        `error: LLM rule '${rule.name}' failed for ${displayStr}: ${error}\n`,
      );
      hasLlmError.value = true;
    } else if (
      response &&
      response.has_violations &&
      response.violations.length > 0
    ) {
      const violation = rule.evaluate(content, response.violations);
      if (violation) {
        diagnostics.push(fromViolation(violation, rule.name, displayStr));
      }
    }
  }
}

export const run = async (): Promise<number> => {
  const program = new Command();

  program
    .name("nori-lint")
    .description("Lint SKILL.md files for common issues")
    .argument("[path]", "Directory to lint", ".")
    .option("--format <format>", "Output format (text or json)", "text")
    .option("--config <path>", "Path to config file")
    .configureOutput({
      writeOut: (str) => process.stdout.write(str),
      writeErr: (str) => process.stderr.write(str),
    })
    .exitOverride();

  let parsedPath: string;
  let format: string;
  let configPath: string | undefined;

  try {
    program.parse(process.argv);
    parsedPath = program.args[0] ?? ".";
    const opts = program.opts<{ format: string; config?: string }>();
    format = opts.format;
    configPath = opts.config;
  } catch {
    return program.args.includes("--help") ? 0 : 1;
  }

  if (format !== "text" && format !== "json") {
    process.stderr.write(
      `error: invalid format '${format}'. Must be 'text' or 'json'\n`,
    );
    return 1;
  }

  let config: Config | null;
  try {
    config = resolveConfig(configPath);
  } catch (e) {
    process.stderr.write(`error: ${e instanceof Error ? e.message : e}\n`);
    return 1;
  }

  const rootPath = parsedPath;

  if (!fs.existsSync(rootPath)) {
    process.stderr.write(`error: ${rootPath} does not exist\n`);
    return 1;
  }

  const stat = fs.statSync(rootPath);
  if (!stat.isDirectory()) {
    process.stderr.write(`error: ${rootPath} is not a directory\n`);
    return 1;
  }

  const registry = defaultRegistry();

  if (!config) {
    process.stderr.write(
      "note: skipping LLM rules (no .nori-lint.json found; use --config to specify)\n",
    );
  }

  const llmClient = config
    ? new AnthropicClient(config.anthropic_api_key)
    : null;
  const llmRegistry = config ? defaultLlmRegistry() : new LlmRegistry();

  if (config) {
    const allKnownRules = [
      ...registry.rules.map((r) => r.name),
      ...llmRegistry.rules.map((r) => r.name),
    ];

    if (config.rules) {
      const namesToCheck = config.rules.enabled ?? config.rules.disabled ?? [];
      for (const name of namesToCheck) {
        if (!allKnownRules.includes(name)) {
          process.stderr.write(`warning: unknown rule '${name}' in config\n`);
        }
      }
    }
  }

  const diagnostics: Array<LintDiagnostic> = [];
  let hasReadError = false;
  const hasLlmError = { value: false };

  const skillFiles = globSync("**/SKILL.md", { cwd: rootPath });

  for (const relPath of skillFiles) {
    const fullPath = path.join(rootPath, relPath);
    const displayPath = relPath;

    let content: string;
    try {
      content = fs.readFileSync(fullPath, "utf-8");
    } catch (e) {
      process.stderr.write(
        `error: could not read ${displayPath}: ${e instanceof Error ? e.message : e}\n`,
      );
      hasReadError = true;
      continue;
    }

    for (const rule of registry.rules) {
      if (config && !isRuleEnabled(config, rule.name)) continue;
      for (const violation of rule.run(content)) {
        diagnostics.push(fromViolation(violation, rule.name, displayPath));
      }
    }

    if (llmClient && config) {
      await runLlmRules(
        llmClient,
        llmRegistry,
        config,
        content,
        displayPath,
        diagnostics,
        hasLlmError,
      );
    }
  }

  const hasViolations =
    diagnostics.length > 0 || hasReadError || hasLlmError.value;

  if (format === "text") {
    for (const diag of diagnostics) {
      if (diag.line !== null) {
        process.stdout.write(
          `[${diag.rule}] ${diag.file}:${diag.line}: ${diag.message}\n`,
        );
      } else {
        process.stdout.write(`[${diag.rule}] ${diag.file}: ${diag.message}\n`);
      }
      if (diag.snippet) {
        process.stdout.write(`  | ${diag.snippet}\n`);
      }
    }
  } else {
    process.stdout.write(JSON.stringify(diagnostics));
  }

  return hasViolations ? 1 : 0;
};

if (import.meta.url === `file://${process.argv[1]}`) {
  run().then((code) => {
    process.exit(code);
  });
}
