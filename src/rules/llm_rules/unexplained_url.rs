use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

const SYSTEM_PROMPT: &str = r#"Find URLs in the provided SKILL.md file that appear without sufficient surrounding context explaining what the linked content is or why it matters. A URL should be accompanied by text that tells the reader what they will find at that URL and why they should care.

Examples of unexplained URLs (BAD):
- "Read https://raw.githubusercontent.com/org/repo/abc123/path/to/file.md" — no description of what the file contains or why to read it
- A bare URL on its own line with no surrounding text
- "See https://example.com/some/deep/path" — the URL path alone is not an explanation

Examples of explained URLs (GOOD — do NOT flag):
- "Read the Rust async book for background on executors: https://rust-lang.github.io/async-book/"
- "The CI configuration is documented at https://docs.github.com/en/actions — specifically the section on workflow triggers"
- URLs inside fenced code blocks (these are code, not documentation references)
- URLs inside command examples like `curl https://api.example.com/v1/health`
- URLs that are clearly self-describing from context (e.g., after a sentence that explains what the linked resource is)

For each violation found, report the unexplained URL as "text" and why it lacks context as "reason"."#;

pub struct UnexplainedUrlRule;

impl LlmRule for UnexplainedUrlRule {
    fn name(&self) -> &str {
        "unexplained_url"
    }

    fn description(&self) -> &str {
        "Checks that URLs in skill files are accompanied by context explaining what they link to"
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn evaluate(&self, _input: &str, violations: &[LlmViolation]) -> Option<RuleViolation> {
        if violations.is_empty() {
            return None;
        }
        let details: Vec<String> = violations
            .iter()
            .map(|u| format!("\"{}\"", u.text))
            .collect();
        Some(RuleViolation {
            message: format!(
                "File contains URLs without surrounding context explaining what they link to: {}",
                details.join(", ")
            ),
            line: None,
            snippet: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::llm_rules::LlmViolation;

    #[test]
    fn has_correct_name() {
        let rule = UnexplainedUrlRule;
        assert_eq!(rule.name(), "unexplained_url");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = UnexplainedUrlRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = UnexplainedUrlRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = UnexplainedUrlRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_unexplained_urls_found() {
        let rule = UnexplainedUrlRule;
        let violations = &[LlmViolation {
            text: "https://raw.githubusercontent.com/org/repo/abc123/path/file.md".into(),
            reason: "No description of what this file contains".into(),
        }];
        let result = rule.evaluate("content with unexplained urls", violations);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("raw.githubusercontent.com"),
            "violation message should include the flagged URL, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_empty_urls_even_if_flagged() {
        let rule = UnexplainedUrlRule;
        let result = rule.evaluate("content", &[]);
        assert!(
            result.is_none(),
            "should return None when no actual URLs are listed"
        );
    }

    #[test]
    fn reports_multiple_unexplained_urls() {
        let rule = UnexplainedUrlRule;
        let violations = &[
            LlmViolation {
                text: "https://example.com/first".into(),
                reason: "no context".into(),
            },
            LlmViolation {
                text: "https://example.com/second".into(),
                reason: "no context".into(),
            },
        ];
        let result = rule.evaluate("content", violations);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(violation.message.contains("example.com/first"));
        assert!(violation.message.contains("example.com/second"));
    }
}
