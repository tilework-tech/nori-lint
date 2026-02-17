import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { loadConfig, isRuleEnabled } from "@/config.js";

import type { Config } from "@/config.js";

/**
 *
 * @param content
 */
function writeTempConfig(content: string): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "nori-lint-test-"));
  const filePath = path.join(dir, "config.json");
  fs.writeFileSync(filePath, content, "utf-8");
  return filePath;
}

describe("loadConfig", () => {
  test("loads valid config with just anthropic_api_key", () => {
    const filePath = writeTempConfig(
      JSON.stringify({ anthropic_api_key: "sk-test-123" }),
    );

    const config = loadConfig(filePath);

    expect(config.anthropic_api_key).toBe("sk-test-123");
  });

  test("returns error for missing file", () => {
    expect(() => loadConfig("/nonexistent/path/config.json")).toThrow();
  });

  test("returns error for malformed JSON", () => {
    const filePath = writeTempConfig("{ not valid json }");

    expect(() => loadConfig(filePath)).toThrow();
  });

  test("returns error for missing anthropic_api_key field", () => {
    const filePath = writeTempConfig(
      JSON.stringify({ some_other_key: "value" }),
    );

    expect(() => loadConfig(filePath)).toThrow();
  });

  test("returns error for empty api key", () => {
    const filePath = writeTempConfig(JSON.stringify({ anthropic_api_key: "" }));

    expect(() => loadConfig(filePath)).toThrow();
  });

  test("returns error for whitespace-only api key", () => {
    const filePath = writeTempConfig(
      JSON.stringify({ anthropic_api_key: "   " }),
    );

    expect(() => loadConfig(filePath)).toThrow();
  });

  test("loads config with disabled rules", () => {
    const filePath = writeTempConfig(
      JSON.stringify({
        anthropic_api_key: "sk-test-123",
        rules: { disabled: ["rule-a", "rule-b"] },
      }),
    );

    const config = loadConfig(filePath);

    expect(config.rules?.disabled).toEqual(["rule-a", "rule-b"]);
  });

  test("loads config with enabled rules", () => {
    const filePath = writeTempConfig(
      JSON.stringify({
        anthropic_api_key: "sk-test-123",
        rules: { enabled: ["rule-a"] },
      }),
    );

    const config = loadConfig(filePath);

    expect(config.rules?.enabled).toEqual(["rule-a"]);
  });

  test("returns error when both enabled and disabled specified", () => {
    const filePath = writeTempConfig(
      JSON.stringify({
        anthropic_api_key: "sk-test-123",
        rules: { enabled: ["rule-a"], disabled: ["rule-b"] },
      }),
    );

    expect(() => loadConfig(filePath)).toThrow();
  });
});

describe("isRuleEnabled", () => {
  test("returns true when no rules config", () => {
    const config: Config = { anthropic_api_key: "sk-test" };

    expect(isRuleEnabled(config, "any-rule")).toBe(true);
  });

  test("returns false for disabled rule", () => {
    const config: Config = {
      anthropic_api_key: "sk-test",
      rules: { disabled: ["no-foo"] },
    };

    expect(isRuleEnabled(config, "no-foo")).toBe(false);
  });

  test("returns true for non-disabled rule", () => {
    const config: Config = {
      anthropic_api_key: "sk-test",
      rules: { disabled: ["no-foo"] },
    };

    expect(isRuleEnabled(config, "other-rule")).toBe(true);
  });

  test("returns true for enabled rule", () => {
    const config: Config = {
      anthropic_api_key: "sk-test",
      rules: { enabled: ["my-rule"] },
    };

    expect(isRuleEnabled(config, "my-rule")).toBe(true);
  });

  test("returns false for non-enabled rule", () => {
    const config: Config = {
      anthropic_api_key: "sk-test",
      rules: { enabled: ["my-rule"] },
    };

    expect(isRuleEnabled(config, "other-rule")).toBe(false);
  });

  test("empty disabled list enables all rules", () => {
    const config: Config = {
      anthropic_api_key: "sk-test",
      rules: { disabled: [] },
    };

    expect(isRuleEnabled(config, "any-rule")).toBe(true);
  });

  test("empty enabled list disables all rules", () => {
    const config: Config = {
      anthropic_api_key: "sk-test",
      rules: { enabled: [] },
    };

    expect(isRuleEnabled(config, "any-rule")).toBe(false);
  });
});
