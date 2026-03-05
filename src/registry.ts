import type { Rule } from "@/rules/index.js";

export class Registry {
  private _rules: Array<Rule> = [];

  register(rule: Rule): void {
    this._rules.push(rule);
  }

  get rules(): Array<Rule> {
    return this._rules;
  }
}
