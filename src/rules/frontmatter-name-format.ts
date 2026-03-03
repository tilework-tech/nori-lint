import { parse } from "yaml";

import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

const MAX_NAME_LENGTH = 64;
const NAME_PATTERN = /^[a-z0-9]+(-[a-z0-9]+)*$/;

function extractName(input: string): string | null {
  const lines = input.split("\n");
  if (lines.length === 0 || lines[0].trim() !== "---") {
    return null;
  }
  let endIdx = -1;
  for (let i = 1; i < lines.length; i++) {
    if (lines[i].trim() === "---") {
      endIdx = i;
      break;
    }
  }
  if (endIdx === -1) return null;

  let data: unknown;
  try {
    data = parse(lines.slice(1, endIdx).join("\n"));
  } catch {
    return null;
  }

  if (data == null || typeof data !== "object" || Array.isArray(data)) {
    return null;
  }

  const fm = data as Record<string, unknown>;
  if (typeof fm.name !== "string" || fm.name.length === 0) {
    return null;
  }

  return fm.name;
}

export const frontmatterNameFormatRule: Rule = {
  name: "frontmatter_name_format",
  description:
    "Checks that the frontmatter name field follows the agentskills.io naming convention: lowercase alphanumeric with hyphens",
  run: (input) => {
    const name = extractName(input);
    if (name === null) return [];

    const violations: Array<RuleViolation> = [];

    if (name.length > MAX_NAME_LENGTH) {
      violations.push({
        message: `Frontmatter name exceeds ${MAX_NAME_LENGTH} characters (got ${name.length})`,
      });
    }

    if (!NAME_PATTERN.test(name)) {
      if (/[A-Z]/.test(name)) {
        violations.push({
          message: "Frontmatter name must use lowercase letters only",
          snippet: name,
        });
      }

      if (name.startsWith("-")) {
        violations.push({
          message: "Frontmatter name must not start with a hyphen",
          snippet: name,
        });
      }

      if (name.endsWith("-")) {
        violations.push({
          message: "Frontmatter name must not end with a hyphen",
          snippet: name,
        });
      }

      if (name.includes("--")) {
        violations.push({
          message: "Frontmatter name must not contain consecutive hyphens",
          snippet: name,
        });
      }

      if (/[^a-zA-Z0-9-]/.test(name)) {
        violations.push({
          message:
            "Frontmatter name may only contain lowercase letters, numbers, and hyphens",
          snippet: name,
        });
      }
    }

    return violations;
  },
};
