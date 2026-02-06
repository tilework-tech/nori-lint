use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, strip_markdown_fences};

const SYSTEM_PROMPT: &str = r#"Analyze the provided SKILL.md file for instructions that tell the reader what NOT to do without providing a corresponding example or explanation of what TO do instead.

A "negative without positive" is when the author says "don't do X" or "avoid X" or "never do X" but fails to follow up with what the reader SHOULD do instead.

<bad_example>
"Don't use global variables."

This tells the reader what to avoid but gives no guidance on the alternative.
</bad_example>

<good_example>
"Don't use global variables. Instead, pass dependencies as function parameters or use dependency injection."

This tells the reader what to avoid AND what to do instead.
</good_example>

<bad_example>
"Avoid writing long functions."

No guidance on what to do instead.
</bad_example>

<good_example>
"Avoid writing long functions. Extract logical sections into well-named helper functions that each do one thing."

Pairs the negative with a concrete positive alternative.
</good_example>

Important: Do NOT flag absolute prohibitions that are safety/policy rules with no reasonable alternative to suggest. For example, "NEVER push to main without permission" or "NEVER delete production data" are absolute guardrails, not instructional guidance — they should NOT be flagged.

Only flag instructional or guidance-style negatives where a positive alternative would genuinely help the reader know what to do.

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "negatives": [{"text": "the exact offending passage", "suggestion": "what positive guidance is missing"}]}

If there are no violations, respond with:
{"has_violations": false, "negatives": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    negatives: Vec<Negative>,
}

#[derive(Deserialize)]
struct Negative {
    text: String,
    suggestion: String,
}

pub struct NegativeWithoutPositiveRule;

impl LlmRule for NegativeWithoutPositiveRule {
    fn name(&self) -> &str {
        "negative_without_positive"
    }

    fn description(&self) -> &str {
        "Checks that instructions saying what NOT to do include a corresponding positive alternative"
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.negatives.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .negatives
            .iter()
            .map(|n| format!("\"{}\" (suggestion: {})", n.text, n.suggestion))
            .collect();

        Some(RuleViolation {
            message: format!(
                "File contains instructions saying what NOT to do without a positive alternative: {}",
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
        let rule = NegativeWithoutPositiveRule;
        assert_eq!(rule.name(), "negative_without_positive");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = NegativeWithoutPositiveRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = NegativeWithoutPositiveRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn system_prompt_asks_for_json_response() {
        let rule = NegativeWithoutPositiveRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("negatives"),
            "system prompt should request JSON with has_violations and negatives fields"
        );
    }

    #[test]
    fn returns_none_when_llm_finds_no_violations() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = r#"{"has_violations": false, "negatives": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_negatives_without_positives() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = r#"{
            "has_violations": true,
            "negatives": [
                {
                    "text": "Don't use global variables.",
                    "suggestion": "Should suggest passing dependencies as function parameters"
                },
                {
                    "text": "Avoid writing long functions.",
                    "suggestion": "Should suggest extracting into smaller helper functions"
                }
            ]
        }"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(
            result.is_some(),
            "should return a violation when negatives are found"
        );
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Don't use global variables."),
            "violation message should include the offending text, got: {}",
            violation.message
        );
        assert!(
            violation
                .message
                .contains("suggestion: Should suggest passing dependencies"),
            "violation message should include the suggestion, got: {}",
            violation.message
        );
        assert!(
            violation.message.contains("Avoid writing long functions."),
            "violation message should include all offending texts, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_violation_with_single_negative() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = r#"{
            "has_violations": true,
            "negatives": [
                {
                    "text": "Never use eval().",
                    "suggestion": "Should suggest safer alternatives like JSON.parse"
                }
            ]
        }"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_some(),
            "should return a violation for a single negative"
        );
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Never use eval()."),
            "violation message should include the offending text, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_malformed_llm_response() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn returns_none_when_has_violations_is_true_but_negatives_empty() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = r#"{"has_violations": true, "negatives": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual negatives are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json_response() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = "```json\n{\"has_violations\": true, \"negatives\": [{\"text\": \"Don't hardcode values.\", \"suggestion\": \"Use constants or config\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_some(),
            "should parse JSON from within markdown fences"
        );
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Don't hardcode values."),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn handles_bare_backtick_fenced_response() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = "```\n{\"has_violations\": false, \"negatives\": []}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should parse JSON from bare backtick fences"
        );
    }

    #[test]
    fn returns_none_when_has_violations_is_false_despite_negatives_populated() {
        let rule = NegativeWithoutPositiveRule;
        let llm_response = r#"{
            "has_violations": false,
            "negatives": [{"text": "Don't do X.", "suggestion": "Do Y instead."}]
        }"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should trust has_violations flag over array contents"
        );
    }
}
