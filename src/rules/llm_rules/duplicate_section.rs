use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

const SYSTEM_PROMPT: &str = r###"Find sections in the provided SKILL.md file that cover substantially the same ground. This includes:

1. Sections with identical or near-identical headings at the same level (e.g., two "## Red Flags" sections)
2. Sections whose content heavily overlaps — they make the same points, give the same warnings, or list the same items even if the headings differ (e.g., "## Common Mistakes" and "## Red Flags" both listing the same anti-patterns)

What NOT to flag:
- A "Quick Reference" or "Checklist" section that intentionally summarizes a longer process section — this is a valid pattern as long as it is clearly labeled as a summary/reference
- Sections that address the same topic but from genuinely different angles (e.g., "## When to Use" and "## When NOT to Use")
- Repeated structure across different phases or steps (e.g., each phase having its own "Examples" subsection is fine)

For each violation found, report both duplicate section headings joined by " / " as "text" and explain why they overlap as "reason".
"###;

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

    fn evaluate(&self, _input: &str, violations: &[LlmViolation]) -> Option<RuleViolation> {
        if violations.is_empty() {
            return None;
        }
        let details: Vec<String> = violations
            .iter()
            .map(|s| format!("\"{}\"", s.text))
            .collect();
        Some(RuleViolation {
            message: format!(
                "File contains sections that cover substantially the same ground: {}",
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
    fn returns_none_when_no_violations() {
        let rule = DuplicateSectionRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_duplicate_sections_found() {
        let rule = DuplicateSectionRule;
        let violations = &[LlmViolation {
            text: "Setup / Installation".into(),
            reason: "both cover initial configuration".into(),
        }];
        let result = rule.evaluate("content with duplicate sections", violations);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Setup / Installation"),
            "violation message should include the section pair, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_empty_sections_even_if_flagged() {
        let rule = DuplicateSectionRule;
        let result = rule.evaluate("content", &[]);
        assert!(
            result.is_none(),
            "should return None when no actual sections are listed"
        );
    }

    #[test]
    fn reports_multiple_duplicate_pairs() {
        let rule = DuplicateSectionRule;
        let violations = &[
            LlmViolation {
                text: "Red Flags / Common Mistakes".into(),
                reason: "overlap".into(),
            },
            LlmViolation {
                text: "Overview / Introduction".into(),
                reason: "overlap".into(),
            },
        ];
        let result = rule.evaluate("content", violations);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(violation.message.contains("Red Flags / Common Mistakes"));
        assert!(violation.message.contains("Overview / Introduction"));
    }
}
