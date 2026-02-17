export type RuleViolation = {
  message: string;
  line?: number;
  snippet?: string;
};

export type LintDiagnostic = {
  rule: string;
  file: string;
  line: number | null;
  snippet: string | null;
  message: string;
};

export const fromViolation = (
  violation: RuleViolation,
  ruleName: string,
  filePath: string,
): LintDiagnostic => {
  return {
    rule: ruleName,
    file: filePath,
    line: violation.line ?? null,
    snippet: violation.snippet ?? null,
    message: violation.message,
  };
};
