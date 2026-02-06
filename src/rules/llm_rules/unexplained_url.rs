use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, strip_markdown_fences};

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

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "urls": [{"url": "the unexplained URL", "reason": "why this URL lacks context"}]}

If there are no violations, respond with:
{"has_violations": false, "urls": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    urls: Vec<UrlViolation>,
}

#[derive(Deserialize)]
struct UrlViolation {
    url: String,
    #[allow(dead_code)]
    reason: String,
}

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

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.urls.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .urls
            .iter()
            .map(|u| format!("\"{}\"", u.url))
            .collect();

        Some(RuleViolation {
            message: format!(
                "File contains URLs without sufficient context: {}",
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
    fn system_prompt_asks_for_json_response() {
        let rule = UnexplainedUrlRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("urls"),
            "system prompt should request JSON with has_violations and urls fields"
        );
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = UnexplainedUrlRule;
        let llm_response = r#"{"has_violations": false, "urls": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_unexplained_urls_found() {
        let rule = UnexplainedUrlRule;
        let llm_response = r#"{
            "has_violations": true,
            "urls": [
                {
                    "url": "https://raw.githubusercontent.com/org/repo/abc123/path/file.md",
                    "reason": "No description of what this file contains"
                }
            ]
        }"#;
        let result = rule.evaluate("content with unexplained urls", llm_response);
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
        let llm_response = r#"{"has_violations": true, "urls": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual URLs are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json() {
        let rule = UnexplainedUrlRule;
        let llm_response = "```json\n{\"has_violations\": true, \"urls\": [{\"url\": \"https://example.com/mystery\", \"reason\": \"no context\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("example.com/mystery"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_malformed_json() {
        let rule = UnexplainedUrlRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn reports_multiple_unexplained_urls() {
        let rule = UnexplainedUrlRule;
        let llm_response = r#"{
            "has_violations": true,
            "urls": [
                {
                    "url": "https://example.com/first",
                    "reason": "no context"
                },
                {
                    "url": "https://example.com/second",
                    "reason": "no context"
                }
            ]
        }"#;
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(violation.message.contains("example.com/first"));
        assert!(violation.message.contains("example.com/second"));
    }
}
