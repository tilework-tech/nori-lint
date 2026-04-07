import pc from "picocolors";

import type { LintDiagnostic } from "@/diagnostic.js";

export type ListRule = {
  name: string;
  description: string;
  type: "deterministic" | "llm";
  fixable: boolean;
};

export function formatDiagnostics(diagnostics: Array<LintDiagnostic>): string {
  if (diagnostics.length === 0) return "";

  const grouped = new Map<string, Array<LintDiagnostic>>();
  for (const diag of diagnostics) {
    const existing = grouped.get(diag.file);
    if (existing) {
      existing.push(diag);
    } else {
      grouped.set(diag.file, [diag]);
    }
  }

  let output = "";

  for (const [file, diags] of grouped) {
    output += `\n  ${pc.cyan(file)}\n`;

    for (const diag of diags) {
      const line = diag.line !== null ? String(diag.line) : "0";
      const location = pc.dim(`${line}:0`);
      const rule = pc.dim(diag.rule);
      output += `    ${location}  ${pc.yellow("warning")}  ${diag.message}  ${rule}\n`;

      if (diag.snippet) {
        output += `${pc.dim("         | " + diag.snippet)}\n`;
      }
    }
  }

  return output;
}

export function formatSummary(count: number): string {
  if (count === 0) {
    return `\n${pc.green("✔ No problems found")}\n`;
  }
  const label = count === 1 ? "problem" : "problems";
  return `\n${pc.red(`✖ ${count} ${label} found`)}\n`;
}

export function formatListRule(rule: ListRule): string {
  const tags = [
    rule.type === "llm" ? pc.cyan("[llm]") : null,
    rule.fixable ? pc.green("[fixable]") : null,
  ]
    .filter(Boolean)
    .join(" ");
  const suffix = tags ? ` ${tags}` : "";
  return `${pc.bold(rule.name)}${suffix}\n  ${pc.dim(rule.description)}\n`;
}

export function formatFixed(filePath: string): string {
  return pc.green(`fixed: ${filePath}`) + "\n";
}

export function formatUnfixable(diag: LintDiagnostic): string {
  return (
    pc.yellow(`[${diag.rule}] ${diag.file}: ${diag.message} (unfixable)`) + "\n"
  );
}

export function formatNote(message: string): string {
  return pc.dim(`note: ${message}`) + "\n";
}

export function formatWarning(message: string): string {
  return pc.yellow(`warning: ${message}`) + "\n";
}

export function formatError(message: string): string {
  return pc.red(`error: ${message}`) + "\n";
}
