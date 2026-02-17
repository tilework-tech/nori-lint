import { processNotIntegrationRule } from "@/rules/llm-rules/process-not-integration.js";

describe("processNotIntegrationRule", () => {
  test("has correct name", () => {
    expect(processNotIntegrationRule.name).toBe("process_not_integration");
  });

  test("has non-empty description", () => {
    expect(processNotIntegrationRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(processNotIntegrationRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = processNotIntegrationRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when process documentation found", () => {
    const result = processNotIntegrationRule.evaluate("content", [
      {
        text: "--name (required): Clear, searchable title",
        reason: "documents API parameters instead of integration",
      },
      {
        text: "Returns confirmation with artifact ID",
        reason: "documents response format instead of workflow",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain(
      "--name (required): Clear, searchable title",
    );
    expect(result!.message).toContain("Returns confirmation with artifact ID");
  });

  test("returns violation with single process evidence", () => {
    const result = processNotIntegrationRule.evaluate("content", [
      {
        text: "POST /api/v1/deploy with payload { branch, env }",
        reason: "raw API call instead of workflow context",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("POST /api/v1/deploy");
  });
});
