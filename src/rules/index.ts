import type { RuleViolation } from "@/diagnostic.js";

export type Rule = {
  name: string;
  description: string;
  run: (input: string) => Array<RuleViolation>;
  fix?: (input: string) => string;
};
