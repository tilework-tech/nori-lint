import { LlmRegistry } from "@/llm-registry.js";

import type { LlmRule } from "@/rules/llm-rules/index.js";

/**
 *
 * @param name
 */
function makeLlmRule(name: string): LlmRule {
  return {
    name,
    description: `Description for ${name}`,
    systemPrompt: `Prompt for ${name}`,
    evaluate: () => null,
  };
}

describe("LlmRegistry", () => {
  test("new registry has no rules", () => {
    const registry = new LlmRegistry();

    expect(registry.rules).toEqual([]);
  });

  test("register adds rule", () => {
    const registry = new LlmRegistry();
    const rule = makeLlmRule("test-llm-rule");

    registry.register(rule);

    expect(registry.rules).toHaveLength(1);
    expect(registry.rules[0].name).toBe("test-llm-rule");
  });

  test("can retrieve registered rules by name", () => {
    const registry = new LlmRegistry();
    registry.register(makeLlmRule("alpha"));
    registry.register(makeLlmRule("beta"));
    registry.register(makeLlmRule("gamma"));

    const names = registry.rules.map((r) => r.name);

    expect(names).toContain("alpha");
    expect(names).toContain("beta");
    expect(names).toContain("gamma");
  });
});
