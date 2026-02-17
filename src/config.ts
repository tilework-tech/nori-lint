import fs from "node:fs";

export type RulesConfig = {
  enabled?: Array<string>;
  disabled?: Array<string>;
};

export type Config = {
  anthropic_api_key: string;
  rules?: RulesConfig;
};

export const loadConfig = (path: string): Config => {
  const raw = fs.readFileSync(path, "utf-8");
  const parsed = JSON.parse(raw) as Record<string, unknown>;

  if (
    !parsed.anthropic_api_key ||
    typeof parsed.anthropic_api_key !== "string"
  ) {
    throw new Error("Config missing required field: anthropic_api_key");
  }

  const apiKey = parsed.anthropic_api_key.trim();
  if (apiKey.length === 0) {
    throw new Error("Config field anthropic_api_key must not be empty");
  }

  const config: Config = { anthropic_api_key: apiKey };

  if (parsed.rules) {
    const rules = parsed.rules as RulesConfig;
    if (rules.enabled && rules.disabled) {
      throw new Error("Config rules cannot specify both enabled and disabled");
    }
    config.rules = rules;
  }

  return config;
};

export const isRuleEnabled = (config: Config, ruleName: string): boolean => {
  if (!config.rules) {
    return true;
  }
  if (config.rules.enabled) {
    return config.rules.enabled.includes(ruleName);
  }
  if (config.rules.disabled) {
    return !config.rules.disabled.includes(ruleName);
  }
  return true;
};
