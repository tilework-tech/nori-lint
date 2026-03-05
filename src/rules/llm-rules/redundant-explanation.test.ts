import { redundantExplanationRule } from "@/rules/llm-rules/redundant-explanation.js";

describe("redundantExplanationRule", () => {
  test("has correct name", () => {
    expect(redundantExplanationRule.name).toBe("redundant_explanation");
  });

  test("has non-empty description", () => {
    expect(redundantExplanationRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(redundantExplanationRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = redundantExplanationRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when redundant explanations found", () => {
    const result = redundantExplanationRule.evaluate("content", [
      {
        text: "GCP stands for Google Cloud Platform",
        reason: "well-known acronym needs no expansion",
      },
      {
        text: "JSON is a lightweight data interchange format",
        reason: "universally known format needs no definition",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("GCP stands for Google Cloud Platform");
    expect(result!.message).toContain(
      "JSON is a lightweight data interchange format",
    );
  });

  test("returns violation with single redundant explanation", () => {
    const result = redundantExplanationRule.evaluate("content", [
      {
        text: "API stands for Application Programming Interface",
        reason: "common acronym",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain(
      "API stands for Application Programming Interface",
    );
  });
});
