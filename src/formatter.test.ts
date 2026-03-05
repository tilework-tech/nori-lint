import { describe, expect, test } from "vitest";

import {
  formatDiagnostics,
  formatSummary,
  formatListRule,
  formatFixed,
  formatUnfixable,
  formatNote,
  formatWarning,
  formatError,
} from "@/formatter.js";

import type { LintDiagnostic } from "@/diagnostic.js";

describe("formatDiagnostics", () => {
  test("groups diagnostics under file headers", () => {
    const diagnostics: Array<LintDiagnostic> = [
      {
        rule: "bold_italics",
        file: "my-skill/SKILL.md",
        line: 7,
        snippet: "Some **bold** content.",
        message: "Bold formatting found: **bold**",
      },
      {
        rule: "line_count",
        file: "my-skill/SKILL.md",
        line: null,
        snippet: null,
        message: "File has 200 lines",
      },
      {
        rule: "unclosed_tags",
        file: "other-skill/SKILL.md",
        line: 3,
        snippet: null,
        message: "Unclosed tag: <required>",
      },
    ];

    const output = formatDiagnostics(diagnostics);

    // Should contain both file headers
    expect(output).toContain("my-skill/SKILL.md");
    expect(output).toContain("other-skill/SKILL.md");

    // Should contain all rule names
    expect(output).toContain("bold_italics");
    expect(output).toContain("line_count");
    expect(output).toContain("unclosed_tags");

    // Should contain all messages
    expect(output).toContain("Bold formatting found: **bold**");
    expect(output).toContain("File has 200 lines");
    expect(output).toContain("Unclosed tag: <required>");

    // Should contain snippet
    expect(output).toContain("Some **bold** content.");
  });

  test("returns empty string for empty diagnostics array", () => {
    const output = formatDiagnostics([]);
    expect(output).toBe("");
  });

  test("handles diagnostics with null line numbers", () => {
    const diagnostics: Array<LintDiagnostic> = [
      {
        rule: "line_count",
        file: "skill/SKILL.md",
        line: null,
        snippet: null,
        message: "File too long",
      },
    ];

    const output = formatDiagnostics(diagnostics);
    expect(output).toContain("line_count");
    expect(output).toContain("File too long");
  });

  test("includes snippet lines under their parent diagnostic", () => {
    const diagnostics: Array<LintDiagnostic> = [
      {
        rule: "bold_italics",
        file: "skill/SKILL.md",
        line: 5,
        snippet: "Some **bold** text",
        message: "Bold found",
      },
    ];

    const output = formatDiagnostics(diagnostics);
    const lines = output.split("\n");

    // Find the line with the message and the line with the snippet
    const msgLineIdx = lines.findIndex((l) => l.includes("Bold found"));
    const snippetLineIdx = lines.findIndex((l) =>
      l.includes("Some **bold** text"),
    );

    expect(msgLineIdx).toBeGreaterThanOrEqual(0);
    expect(snippetLineIdx).toBeGreaterThan(msgLineIdx);
  });

  test("omits snippet line when snippet is null", () => {
    const diagnostics: Array<LintDiagnostic> = [
      {
        rule: "line_count",
        file: "skill/SKILL.md",
        line: null,
        snippet: null,
        message: "File too long",
      },
    ];

    const output = formatDiagnostics(diagnostics);
    const lines = output.split("\n");
    expect(lines.some((l) => l.trimStart().startsWith("|"))).toBe(false);
  });
});

describe("formatSummary", () => {
  test("shows problem count when count > 0", () => {
    const output = formatSummary(5);
    expect(output).toContain("5");
    expect(output).toContain("problem");
  });

  test("shows success indicator when count is 0", () => {
    const output = formatSummary(0);
    expect(output).toContain("✔");
    expect(output).toContain("No problems");
  });
});

describe("formatListRule", () => {
  test("includes rule name and description", () => {
    const output = formatListRule({
      name: "bold_italics",
      description: "Checks for bold formatting",
      type: "deterministic" as const,
      fixable: false,
    });
    expect(output).toContain("bold_italics");
    expect(output).toContain("Checks for bold formatting");
  });

  test("includes llm tag for llm rules", () => {
    const output = formatListRule({
      name: "first_person",
      description: "Checks for first person",
      type: "llm" as const,
      fixable: true,
    });
    expect(output).toContain("[llm]");
  });

  test("includes fixable tag for fixable rules", () => {
    const output = formatListRule({
      name: "bold_italics",
      description: "Checks for bold",
      type: "deterministic" as const,
      fixable: true,
    });
    expect(output).toContain("[fixable]");
  });

  test("omits tags for non-llm non-fixable rules", () => {
    const output = formatListRule({
      name: "line_count",
      description: "Checks line count",
      type: "deterministic" as const,
      fixable: false,
    });
    expect(output).not.toContain("[llm]");
    expect(output).not.toContain("[fixable]");
  });
});

describe("formatFixed", () => {
  test("includes the file path", () => {
    const output = formatFixed("my-skill/SKILL.md");
    expect(output).toContain("my-skill/SKILL.md");
    expect(output).toContain("fixed");
  });
});

describe("formatUnfixable", () => {
  test("includes rule name, file, and message", () => {
    const diag: LintDiagnostic = {
      rule: "line_count",
      file: "my-skill/SKILL.md",
      line: null,
      snippet: null,
      message: "File has 200 lines",
    };
    const output = formatUnfixable(diag);
    expect(output).toContain("line_count");
    expect(output).toContain("my-skill/SKILL.md");
    expect(output).toContain("File has 200 lines");
    expect(output).toContain("unfixable");
  });
});

describe("formatNote", () => {
  test("includes the message", () => {
    const output = formatNote("skipping LLM rules");
    expect(output).toContain("skipping LLM rules");
  });
});

describe("formatWarning", () => {
  test("includes the message", () => {
    const output = formatWarning("unknown rule 'foo'");
    expect(output).toContain("unknown rule 'foo'");
  });
});

describe("formatError", () => {
  test("includes the message", () => {
    const output = formatError("could not read file");
    expect(output).toContain("could not read file");
  });
});
