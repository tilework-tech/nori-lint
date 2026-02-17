import type { RuleViolation } from "@/diagnostic.js";

export type LlmResponse = {
  has_violations: boolean;
  violations: Array<LlmViolation>;
};

export type LlmViolation = {
  text: string;
  reason: string;
};

export type LlmRule = {
  name: string;
  description: string;
  systemPrompt: string;
  evaluate: (
    input: string,
    violations: Array<LlmViolation>,
  ) => RuleViolation | null;
};
