use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, strip_markdown_fences};

const SYSTEM_PROMPT: &str = r###"Find sections in the provided SKILL.md file that cover substantially the same ground. This includes:

1. Sections with identical or near-identical headings at the same level (e.g., two "## Red Flags" sections)
2. Sections whose content heavily overlaps — they make the same points, give the same warnings, or list the same items even if the headings differ (e.g., "## Common Mistakes" and "## Red Flags" both listing the same anti-patterns)

What NOT to flag:
- A "Quick Reference" or "Checklist" section that intentionally summarizes a longer process section — this is a valid pattern as long as it is clearly labeled as a summary/reference
- Sections that address the same topic but from genuinely different angles (e.g., "## When to Use" and "## When NOT to Use")
- Repeated structure across different phases or steps (e.g., each phase having its own "Examples" subsection is fine)

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "sections": [{"heading_a": "first section heading", "heading_b": "second section heading", "reason": "why these are duplicates"}]}

If there are no violations, respond with:
{"has_violations": false, "sections": []}
"###;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    sections: Vec<SectionPair>,
}

#[derive(Deserialize)]
struct SectionPair {
    heading_a: String,
    heading_b: String,
    #[allow(dead_code)]
    reason: String,
}

pub struct DuplicateSectionRule;

impl LlmRule for DuplicateSectionRule {
    fn name(&self) -> &str {
        "duplicate_section"
    }

    fn description(&self) -> &str {
        "Checks that skill files do not contain sections that cover substantially the same ground"
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.sections.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .sections
            .iter()
            .map(|s| format!("\"{}\" and \"{}\"", s.heading_a, s.heading_b))
            .collect();

        Some(RuleViolation {
            message: format!(
                "File contains sections that cover the same ground: {}",
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
        let rule = DuplicateSectionRule;
        assert_eq!(rule.name(), "duplicate_section");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = DuplicateSectionRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = DuplicateSectionRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn system_prompt_asks_for_json_response() {
        let rule = DuplicateSectionRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("sections"),
            "system prompt should request JSON with has_violations and sections fields"
        );
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = DuplicateSectionRule;
        let llm_response = r#"{"has_violations": false, "sections": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_duplicate_sections_found() {
        let rule = DuplicateSectionRule;
        let llm_response = "{\"has_violations\": true, \"sections\": [{\"heading_a\": \"Red Flags\", \"heading_b\": \"Common Mistakes\", \"reason\": \"Both sections list the same anti-patterns\"}]}";
        let result = rule.evaluate("content with duplicate sections", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Red Flags"),
            "violation message should include first heading, got: {}",
            violation.message
        );
        assert!(
            violation.message.contains("Common Mistakes"),
            "violation message should include second heading, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_empty_sections_even_if_flagged() {
        let rule = DuplicateSectionRule;
        let llm_response = r#"{"has_violations": true, "sections": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual sections are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json() {
        let rule = DuplicateSectionRule;
        let llm_response = "```json\n{\"has_violations\": true, \"sections\": [{\"heading_a\": \"Overview\", \"heading_b\": \"Summary\", \"reason\": \"same content\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Overview"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_malformed_json() {
        let rule = DuplicateSectionRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn reports_multiple_duplicate_pairs() {
        let rule = DuplicateSectionRule;
        let llm_response = "{\"has_violations\": true, \"sections\": [{\"heading_a\": \"Red Flags\", \"heading_b\": \"Common Mistakes\", \"reason\": \"overlap\"}, {\"heading_a\": \"Overview\", \"heading_b\": \"Introduction\", \"reason\": \"overlap\"}]}";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(violation.message.contains("Red Flags"));
        assert!(violation.message.contains("Overview"));
    }
}
