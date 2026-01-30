use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleViolation {
    pub message: String,
    pub line: Option<u32>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintDiagnostic {
    pub rule: String,
    pub file: String,
    pub line: Option<u32>,
    pub snippet: Option<String>,
    pub message: String,
}

impl LintDiagnostic {
    pub fn from_violation(violation: &RuleViolation, rule_name: &str, file_path: &str) -> Self {
        Self {
            rule: rule_name.to_string(),
            file: file_path.to_string(),
            line: violation.line,
            snippet: violation.snippet.clone(),
            message: violation.message.clone(),
        }
    }
}
