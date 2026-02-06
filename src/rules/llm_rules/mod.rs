pub mod cli_command_index;
pub mod redundant_explanation;

use crate::diagnostic::RuleViolation;

pub trait LlmRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn evaluate(&self, input: &str, llm_response: &str) -> Option<RuleViolation>;
}

pub fn strip_markdown_fences(s: &str) -> &str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_fences_returns_plain_text_unchanged() {
        assert_eq!(strip_markdown_fences("hello world"), "hello world");
    }

    #[test]
    fn strip_fences_removes_json_fences() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn strip_fences_removes_bare_fences() {
        let input = "```\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn strip_fences_handles_surrounding_whitespace() {
        let input = "  \n```json\n{\"key\": \"value\"}\n```\n  ";
        assert_eq!(strip_markdown_fences(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn strip_fences_handles_missing_closing_fence() {
        let input = "```json\n{\"key\": \"value\"}";
        assert_eq!(strip_markdown_fences(input), "{\"key\": \"value\"}");
    }
}
