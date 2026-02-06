pub mod first_person;
pub mod redundant_explanation;

use crate::diagnostic::RuleViolation;

pub trait LlmRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn evaluate(&self, input: &str, llm_response: &str) -> Option<RuleViolation>;
}

pub(crate) fn strip_markdown_fences(s: &str) -> &str {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let inner = if let Some(after_lang) = rest.split_once('\n') {
            after_lang.1
        } else {
            rest
        };
        inner.strip_suffix("```").unwrap_or(inner).trim()
    } else {
        trimmed
    }
}
