import { describe, it, expect } from "vitest";

import {
  defaultRegistry,
  defaultLlmRegistry,
  isRuleEnabled,
  fromViolation,
} from "@/index.js";

import type { Config, LintDiagnostic } from "@/index.js";

describe("library exports", () => {
  describe("defaultRegistry", () => {
    it("returns a registry whose rules can lint content", () => {
      const registry = defaultRegistry();
      const content =
        "---\nname: test\ndescription: Use when testing\n---\n\nhello   \n";

      const diagnostics: Array<LintDiagnostic> = [];
      for (const rule of registry.rules) {
        for (const violation of rule.run(content)) {
          diagnostics.push(fromViolation(violation, rule.name, "test.md"));
        }
      }

      expect(diagnostics.length).toBeGreaterThan(0);
      expect(diagnostics.some((d) => d.rule === "trailing_whitespace")).toBe(
        true,
      );
    });
  });

  describe("defaultLlmRegistry", () => {
    it("returns a registry with LLM rules", () => {
      const registry = defaultLlmRegistry();
      expect(registry.rules.length).toBeGreaterThan(0);
      for (const rule of registry.rules) {
        expect(rule.name).toBeTruthy();
        expect(rule.systemPrompt).toBeTruthy();
        expect(typeof rule.evaluate).toBe("function");
      }
    });
  });

  describe("config filtering", () => {
    it("isRuleEnabled respects disabled list", () => {
      const config: Config = {
        anthropic_api_key: "test-key",
        rules: { disabled: ["trailing_whitespace"] },
      };
      expect(isRuleEnabled(config, "trailing_whitespace")).toBe(false);
      expect(isRuleEnabled(config, "bold_italics")).toBe(true);
    });

    it("isRuleEnabled respects enabled list", () => {
      const config: Config = {
        anthropic_api_key: "test-key",
        rules: { enabled: ["trailing_whitespace"] },
      };
      expect(isRuleEnabled(config, "trailing_whitespace")).toBe(true);
      expect(isRuleEnabled(config, "bold_italics")).toBe(false);
    });
  });
});
