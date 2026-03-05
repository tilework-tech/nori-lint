import { linkableContentRule } from "@/rules/llm-rules/linkable-content.js";

describe("linkableContentRule", () => {
  test("has correct name", () => {
    expect(linkableContentRule.name).toBe("linkable_content");
  });

  test("has non-empty description", () => {
    expect(linkableContentRule.description.length).toBeGreaterThan(0);
  });

  test("has non-empty system prompt", () => {
    expect(linkableContentRule.systemPrompt.length).toBeGreaterThan(0);
  });

  test("returns null when no violations", () => {
    const result = linkableContentRule.evaluate("some content", []);
    expect(result).toBeNull();
  });

  test("returns violation when linkable content found", () => {
    const result = linkableContentRule.evaluate("content", [
      {
        text: "GCP Cloud Functions can be configured by setting the runtime field in your cloudfunctions.json",
        reason:
          "Consider linking to: https://cloud.google.com/functions/docs/configuring",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("GCP Cloud Functions");
  });

  test("returns violation with multiple linkable sections", () => {
    const result = linkableContentRule.evaluate("content", [
      {
        text: "Docker containers use a layered filesystem",
        reason:
          "Consider linking to: https://docs.docker.com/storage/storagedriver/",
      },
      {
        text: "GitHub Actions workflows are defined in YAML files under .github/workflows",
        reason:
          "Consider linking to: https://docs.github.com/en/actions/using-workflows",
      },
    ]);
    expect(result).not.toBeNull();
    expect(result!.message).toContain("Docker containers");
    expect(result!.message).toContain("GitHub Actions workflows");
  });
});
