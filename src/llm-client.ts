import type { LlmResponse, LlmViolation } from "@/rules/llm-rules/index.js";

const ANTHROPIC_API_URL = "https://api.anthropic.com/v1/messages";
const MODEL = "claude-opus-4-5-20251101";
const MAX_TOKENS = 1024;
const REQUEST_TIMEOUT = 60000;
const TOOL_NAME = "report_lint_violations";

export class LlmError extends Error {
  constructor(
    public type: "http" | "parse",
    message: string,
  ) {
    super(message);
    this.name = "LlmError";
  }
}

export type LlmFixViolation = {
  rule: string;
  message: string;
};

export type LlmAnalyzer = {
  analyze: (systemPrompt: string, userContent: string) => Promise<LlmResponse>;
  fixContent: (
    fileContent: string,
    violations: Array<LlmFixViolation>,
  ) => Promise<string>;
};

export const extractToolInputFromResponse = (body: unknown): LlmResponse => {
  const obj = body as Record<string, unknown>;
  const content = obj.content as Array<Record<string, unknown>> | undefined;
  if (!content || !Array.isArray(content)) {
    throw new LlmError("parse", "missing content array in API response");
  }

  const toolBlock = content.find((block) => block.type === "tool_use");
  if (!toolBlock) {
    throw new LlmError("parse", "no tool_use block in API response");
  }

  const input = toolBlock.input as Record<string, unknown>;
  if (typeof input.has_violations !== "boolean") {
    throw new LlmError(
      "parse",
      "failed to parse tool input: has_violations must be boolean",
    );
  }

  if (!Array.isArray(input.violations)) {
    throw new LlmError(
      "parse",
      "failed to parse tool input: violations must be array",
    );
  }

  return {
    has_violations: input.has_violations as boolean,
    violations: input.violations as Array<LlmViolation>,
  };
};

export const extractFixFromResponse = (body: unknown): string => {
  const obj = body as Record<string, unknown>;
  const content = obj.content as Array<Record<string, unknown>> | undefined;
  if (!content || !Array.isArray(content)) {
    throw new LlmError("parse", "missing content array in API response");
  }

  const toolBlock = content.find((block) => block.type === "tool_use");
  if (!toolBlock) {
    throw new LlmError("parse", "no tool_use block in API response");
  }

  const input = toolBlock.input as Record<string, unknown>;
  if (typeof input.fixed_content !== "string") {
    throw new LlmError(
      "parse",
      "failed to parse tool input: fixed_content must be a string",
    );
  }

  return input.fixed_content as string;
};

const FIX_TOOL_NAME = "apply_fixes";
const FIX_MAX_TOKENS = 8192;

export class AnthropicClient implements LlmAnalyzer {
  private apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async analyze(
    systemPrompt: string,
    userContent: string,
  ): Promise<LlmResponse> {
    const body = {
      model: MODEL,
      max_tokens: MAX_TOKENS,
      system: systemPrompt,
      messages: [{ role: "user", content: userContent }],
      tools: [
        {
          name: TOOL_NAME,
          description: "Report lint violations found in the file",
          input_schema: {
            type: "object",
            properties: {
              has_violations: { type: "boolean" },
              violations: {
                type: "array",
                items: {
                  type: "object",
                  properties: {
                    text: { type: "string", description: "The offending text" },
                    reason: {
                      type: "string",
                      description: "Why this is a violation",
                    },
                  },
                  required: ["text", "reason"],
                },
              },
            },
            required: ["has_violations", "violations"],
          },
        },
      ],
      tool_choice: { type: "tool", name: TOOL_NAME },
    };

    const response = await fetch(ANTHROPIC_API_URL, {
      method: "POST",
      headers: {
        "x-api-key": this.apiKey,
        "anthropic-version": "2023-06-01",
        "content-type": "application/json",
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(REQUEST_TIMEOUT),
    });

    if (!response.ok) {
      const bodyText = await response.text();
      throw new LlmError(
        "http",
        `API returned status ${response.status}: ${bodyText}`,
      );
    }

    const responseBody: unknown = await response.json();
    return extractToolInputFromResponse(responseBody);
  }

  async fixContent(
    fileContent: string,
    violations: Array<LlmFixViolation>,
  ): Promise<string> {
    const violationList = violations
      .map((v) => `- [${v.rule}] ${v.message}`)
      .join("\n");

    const systemPrompt = `You are fixing a SKILL.md file based on lint violations found by a linter.

Rules:
- ONLY modify the specific text cited in each violation. Do not touch anything else.
- NEVER remove or modify lines, sections, or content that are not directly cited in a violation.
- Do not add new content.
- Preserve the file's structure, meaning, and whitespace.`;

    const userMessage = `Here is the file content:\n\n${fileContent}\n\nThe following violations were found:\n${violationList}\n\nFix all violations and return the corrected file content.`;

    const body = {
      model: MODEL,
      max_tokens: FIX_MAX_TOKENS,
      system: systemPrompt,
      messages: [{ role: "user", content: userMessage }],
      tools: [
        {
          name: FIX_TOOL_NAME,
          description: "Return the fixed file content",
          input_schema: {
            type: "object",
            properties: {
              fixed_content: {
                type: "string",
                description: "The complete fixed file content",
              },
            },
            required: ["fixed_content"],
          },
        },
      ],
      tool_choice: { type: "tool", name: FIX_TOOL_NAME },
    };

    const response = await fetch(ANTHROPIC_API_URL, {
      method: "POST",
      headers: {
        "x-api-key": this.apiKey,
        "anthropic-version": "2023-06-01",
        "content-type": "application/json",
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(REQUEST_TIMEOUT),
    });

    if (!response.ok) {
      const bodyText = await response.text();
      throw new LlmError(
        "http",
        `API returned status ${response.status}: ${bodyText}`,
      );
    }

    const responseBody: unknown = await response.json();
    const obj = responseBody as Record<string, unknown>;
    if (obj.stop_reason === "max_tokens") {
      throw new LlmError(
        "parse",
        "LLM response was truncated (max_tokens reached); fix may be incomplete",
      );
    }
    return extractFixFromResponse(responseBody);
  }
}
