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

export type LlmAnalyzer = {
  analyze: (systemPrompt: string, userContent: string) => Promise<LlmResponse>;
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
}
