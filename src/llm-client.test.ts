import {
  extractToolInputFromResponse,
  extractFixFromResponse,
} from "@/llm-client.js";

describe("extractToolInputFromResponse", () => {
  test("extracts violations from tool_use response", () => {
    const body = {
      content: [
        {
          type: "tool_use",
          id: "tool_123",
          name: "lint_response",
          input: {
            has_violations: true,
            violations: [
              { text: "Use active voice", reason: "Passive voice detected" },
            ],
          },
        },
      ],
    };

    const result = extractToolInputFromResponse(body);

    expect(result.has_violations).toBe(true);
    expect(result.violations).toHaveLength(1);
    expect(result.violations[0].text).toBe("Use active voice");
    expect(result.violations[0].reason).toBe("Passive voice detected");
  });

  test("extracts no violations when has_violations is false", () => {
    const body = {
      content: [
        {
          type: "tool_use",
          id: "tool_456",
          name: "lint_response",
          input: {
            has_violations: false,
            violations: [],
          },
        },
      ],
    };

    const result = extractToolInputFromResponse(body);

    expect(result.has_violations).toBe(false);
    expect(result.violations).toEqual([]);
  });

  test("returns error for missing tool_use block", () => {
    const body = {
      content: [
        {
          type: "text",
          text: "No tool use here",
        },
      ],
    };

    expect(() => extractToolInputFromResponse(body)).toThrow();
  });

  test("returns error for empty content array", () => {
    const body = {
      content: [],
    };

    expect(() => extractToolInputFromResponse(body)).toThrow();
  });

  test("returns error for missing content field", () => {
    const body = {};

    expect(() => extractToolInputFromResponse(body)).toThrow();
  });

  test("returns error for malformed tool input", () => {
    const body = {
      content: [
        {
          type: "tool_use",
          id: "tool_789",
          name: "lint_response",
          input: {
            has_violations: "yes",
            violations: [],
          },
        },
      ],
    };

    expect(() => extractToolInputFromResponse(body)).toThrow();
  });

  test("extracts tool_use when mixed with text blocks", () => {
    const body = {
      content: [
        {
          type: "text",
          text: "Here is some preamble text",
        },
        {
          type: "tool_use",
          id: "tool_abc",
          name: "lint_response",
          input: {
            has_violations: true,
            violations: [{ text: "Bad pattern", reason: "Violates guideline" }],
          },
        },
      ],
    };

    const result = extractToolInputFromResponse(body);

    expect(result.has_violations).toBe(true);
    expect(result.violations).toHaveLength(1);
    expect(result.violations[0].text).toBe("Bad pattern");
  });

  test("returns no-violations fallback when allowMissing is true and no tool_use block", () => {
    const body = {
      content: [
        {
          type: "server_tool_use",
          id: "srvtool_1",
          name: "web_search",
          input: { query: "docker docs" },
        },
        {
          type: "web_search_tool_result",
          tool_use_id: "srvtool_1",
          content: [
            { type: "web_search_result", url: "https://docs.docker.com" },
          ],
        },
        {
          type: "text",
          text: "No violations found.",
        },
      ],
    };

    const result = extractToolInputFromResponse(body, true);

    expect(result.has_violations).toBe(false);
    expect(result.violations).toEqual([]);
  });

  test("still throws when allowMissing is false and no tool_use block", () => {
    const body = {
      content: [{ type: "text", text: "No tool use" }],
    };

    expect(() => extractToolInputFromResponse(body, false)).toThrow();
  });

  test("extracts tool_use even when allowMissing is true and tool_use exists", () => {
    const body = {
      content: [
        {
          type: "server_tool_use",
          id: "srvtool_2",
          name: "web_search",
          input: { query: "github actions docs" },
        },
        {
          type: "web_search_tool_result",
          tool_use_id: "srvtool_2",
          content: [],
        },
        {
          type: "tool_use",
          id: "tool_xyz",
          name: "report_lint_violations",
          input: {
            has_violations: true,
            violations: [
              {
                text: "Inlined docs",
                reason: "Link to https://docs.github.com",
              },
            ],
          },
        },
      ],
    };

    const result = extractToolInputFromResponse(body, true);

    expect(result.has_violations).toBe(true);
    expect(result.violations).toHaveLength(1);
    expect(result.violations[0].text).toBe("Inlined docs");
  });
});

describe("extractFixFromResponse", () => {
  test("extracts fixed_content from tool_use response", () => {
    const body = {
      content: [
        {
          type: "tool_use",
          id: "tool_fix_1",
          name: "apply_fixes",
          input: {
            fixed_content: "The corrected file content here.",
          },
        },
      ],
    };

    const result = extractFixFromResponse(body);
    expect(result).toBe("The corrected file content here.");
  });

  test("throws for missing tool_use block", () => {
    const body = {
      content: [{ type: "text", text: "No tool use" }],
    };

    expect(() => extractFixFromResponse(body)).toThrow();
  });

  test("throws for missing fixed_content field", () => {
    const body = {
      content: [
        {
          type: "tool_use",
          id: "tool_fix_2",
          name: "apply_fixes",
          input: { something_else: "wrong field" },
        },
      ],
    };

    expect(() => extractFixFromResponse(body)).toThrow();
  });

  test("throws for non-string fixed_content", () => {
    const body = {
      content: [
        {
          type: "tool_use",
          id: "tool_fix_3",
          name: "apply_fixes",
          input: { fixed_content: 42 },
        },
      ],
    };

    expect(() => extractFixFromResponse(body)).toThrow();
  });

  test("extracts fix when mixed with text blocks", () => {
    const body = {
      content: [
        { type: "text", text: "preamble" },
        {
          type: "tool_use",
          id: "tool_fix_4",
          name: "apply_fixes",
          input: { fixed_content: "Fixed content here." },
        },
      ],
    };

    const result = extractFixFromResponse(body);
    expect(result).toBe("Fixed content here.");
  });
});
