import type { LlmRule } from "@/rules/llm-rules/index.js";

export class LlmRegistry {
  private _rules: Array<LlmRule> = [];

  register(rule: LlmRule): void {
    this._rules.push(rule);
  }

  get rules(): Array<LlmRule> {
    return this._rules;
  }
}
