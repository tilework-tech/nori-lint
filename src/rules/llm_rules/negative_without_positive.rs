use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

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

For each violation found, report the negative instruction as "text" and what positive alternative is missing as "reason"."#;

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

    fn evaluate(&self, _input: &str, violations: &[LlmViolation]) -> Option<RuleViolation> {
        if violations.is_empty() {
            return None;
        }
        let details: Vec<String> = violations
            .iter()
            .map(|v| format!("\"{}\" (suggestion: {})", v.text, v.reason))
            .collect();
        Some(RuleViolation {
            message: format!(
                "File contains negative instructions without positive alternatives: {}",
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
    fn returns_none_when_no_violations() {
        let rule = NegativeWithoutPositiveRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_negatives_without_positives() {
        let rule = NegativeWithoutPositiveRule;
        let result = rule.evaluate(
            "some skill content",
            &[
                LlmViolation {
                    text: "Don't use global variables.".into(),
                    reason: "Should suggest passing dependencies as function parameters".into(),
                },
                LlmViolation {
                    text: "Avoid writing long functions.".into(),
                    reason: "Should suggest extracting into smaller helper functions".into(),
                },
            ],
        );
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
        let result = rule.evaluate(
            "content",
            &[LlmViolation {
                text: "Never use eval().".into(),
                reason: "Should suggest safer alternatives like JSON.parse".into(),
            }],
        );
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
}
