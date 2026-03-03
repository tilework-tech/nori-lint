import { parse } from "yaml";

import type { RuleViolation } from "@/diagnostic.js";
import type { Rule } from "@/rules/index.js";

const KNOWN_FIELDS = new Set([
  "name",
  "description",
  "license",
  "compatibility",
  "metadata",
  "allowed-tools",
]);

const MAX_DESCRIPTION_LENGTH = 1024;
const MAX_COMPATIBILITY_LENGTH = 500;

function extractFrontmatter(input: string): { yaml: string; found: boolean } {
  const lines = input.split("\n");
  if (lines.length === 0 || lines[0].trim() !== "---") {
    return { yaml: "", found: false };
  }
  for (let i = 1; i < lines.length; i++) {
    if (lines[i].trim() === "---") {
      return { yaml: lines.slice(1, i).join("\n"), found: true };
    }
  }
  return { yaml: "", found: false };
}

export const frontmatterRule: Rule = {
  name: "frontmatter",
  description:
    "Checks that SKILL.md files have valid YAML frontmatter with required fields per the agentskills.io specification",
  run: (input) => {
    const violations: Array<RuleViolation> = [];

    const { yaml: yamlStr, found } = extractFrontmatter(input);
    if (!found) {
      return [
        {
          message: "File is missing YAML frontmatter (expected --- delimiters)",
        },
      ];
    }

    let data: unknown;
    try {
      data = parse(yamlStr);
    } catch {
      return [{ message: "Frontmatter contains invalid YAML" }];
    }

    if (data == null || typeof data !== "object" || Array.isArray(data)) {
      return [
        { message: "Missing required frontmatter field: name" },
        { message: "Missing required frontmatter field: description" },
      ];
    }

    const fm = data as Record<string, unknown>;

    if (
      fm.name === undefined ||
      fm.name === null ||
      typeof fm.name !== "string" ||
      fm.name.length === 0
    ) {
      violations.push({
        message: "Missing required frontmatter field: name",
      });
    }

    if (
      fm.description === undefined ||
      fm.description === null ||
      typeof fm.description !== "string" ||
      fm.description.length === 0
    ) {
      violations.push({
        message: "Missing required frontmatter field: description",
      });
    }

    if (
      typeof fm.description === "string" &&
      fm.description.length > MAX_DESCRIPTION_LENGTH
    ) {
      violations.push({
        message: `Frontmatter description exceeds ${MAX_DESCRIPTION_LENGTH} characters (got ${fm.description.length})`,
      });
    }

    if (
      fm.compatibility !== undefined &&
      typeof fm.compatibility === "string" &&
      fm.compatibility.length > MAX_COMPATIBILITY_LENGTH
    ) {
      violations.push({
        message: `Frontmatter compatibility exceeds ${MAX_COMPATIBILITY_LENGTH} characters (got ${fm.compatibility.length})`,
      });
    }

    if (fm.metadata !== undefined) {
      if (
        typeof fm.metadata !== "object" ||
        fm.metadata === null ||
        Array.isArray(fm.metadata)
      ) {
        violations.push({
          message:
            "Frontmatter metadata must be a key-value map, not a scalar or array",
        });
      }
    }

    for (const key of Object.keys(fm)) {
      if (!KNOWN_FIELDS.has(key)) {
        violations.push({
          message: `Unknown frontmatter field: ${key}`,
        });
      }
    }

    return violations;
  },
};
