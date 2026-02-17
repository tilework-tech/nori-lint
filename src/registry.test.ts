import { Registry } from "@/registry.js";

import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

describe("Registry", () => {
  test("new registry has no rules", () => {
    const registry = new Registry();

    expect(registry.rules).toEqual([]);
  });

  test("register adds rule to registry", () => {
    const registry = new Registry();
    const rule: Rule = {
      name: "test-rule",
      description: "A test rule",
      run: () => [],
    };

    registry.register(rule);

    expect(registry.rules).toHaveLength(1);
    expect(registry.rules[0].name).toBe("test-rule");
  });

  test("registered rules run correctly", () => {
    const registry = new Registry();

    const passingRule: Rule = {
      name: "passing-rule",
      description: "Always passes",
      run: () => [],
    };

    const failingRule: Rule = {
      name: "failing-rule",
      description: "Always fails",
      run: (): Array<RuleViolation> => [
        { message: "something is wrong", line: 10, snippet: "bad code" },
      ],
    };

    registry.register(passingRule);
    registry.register(failingRule);

    expect(registry.rules).toHaveLength(2);

    const passingResult = registry.rules[0].run("some input");
    expect(passingResult).toEqual([]);

    const failingResult = registry.rules[1].run("some input");
    expect(failingResult).toHaveLength(1);
    expect(failingResult[0].message).toBe("something is wrong");
  });
});
