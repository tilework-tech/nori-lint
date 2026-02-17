import { extractToolInputFromResponse } from "@/llm-client.js";

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
});
