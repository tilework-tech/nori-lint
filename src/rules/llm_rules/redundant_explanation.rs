use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

const SYSTEM_PROMPT: &str = r#"Find passages in the provided file where the author wastes tokens explaining concepts that a modern LLM already knows from its training data.

<bad_example>
"GCP stands for Google Cloud Platform"
"JSON is a lightweight data interchange format"
"REST is a style of web API design"
"Git is a version control system"
</bad_example>

These lines should just be removed.

Sometimes this can be subtle.

<bad_example>
7. Main branches are often protected. This is done to ensure that we do not accidentally deploy things to production that contain bugs. Confirm that you are not on the main branch. If you are, ask me before proceeding. NEVER push to main without permission.
</bad_example>

<good_example>
7. Confirm that you are not on the main branch. If you are, ask me before proceeding. NEVER push to main without permission.
</good_example>

<bad_example>
Sometimes there can be conflicts between the branch you are merging and the main branch. Merge main and resolve conflicts if necessary.
</bad_example>

<good_example>
Sometimes there can be conflicts between the branch you are merging and the main branch. Merge main and resolve conflicts if necessary.
</good_example>

In the above examples, the author explains how main branch protection works and that merge conflicts can exist. These are redundant and should be removed.

For each violation found, report the redundant explanation as "text" and why it is redundant as "reason"."#;

pub struct RedundantExplanationRule;

impl LlmRule for RedundantExplanationRule {
    fn name(&self) -> &str {
        "redundant_explanation"
    }

    fn description(&self) -> &str {
        "Checks that skill files do not waste tokens explaining concepts an LLM already knows"
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
            .map(|e| format!("\"{}\"", e.text))
            .collect();
        Some(RuleViolation {
            message: format!(
                "File contains explanations of concepts the LLM already knows: {}",
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
        let rule = RedundantExplanationRule;
        assert_eq!(rule.name(), "redundant_explanation");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = RedundantExplanationRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = RedundantExplanationRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = RedundantExplanationRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_redundant_explanations() {
        let rule = RedundantExplanationRule;
        let result = rule.evaluate(
            "some skill content with explanations",
            &[
                LlmViolation {
                    text: "GCP stands for Google Cloud Platform".into(),
                    reason: "LLMs are trained on extensive documentation about GCP".into(),
                },
                LlmViolation {
                    text: "JSON is a lightweight data interchange format".into(),
                    reason: "JSON is fundamental knowledge for any LLM".into(),
                },
            ],
        );
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation
                .message
                .contains("GCP stands for Google Cloud Platform"),
            "violation message should include the redundant text, got: {}",
            violation.message
        );
        assert!(
            violation
                .message
                .contains("JSON is a lightweight data interchange format"),
            "violation message should include all redundant texts, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_violation_with_single_explanation() {
        let rule = RedundantExplanationRule;
        let result = rule.evaluate(
            "content",
            &[LlmViolation {
                text: "REST stands for Representational State Transfer".into(),
                reason: "Well-known concept".into(),
            }],
        );
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation
                .message
                .contains("REST stands for Representational State Transfer"),
            "violation message should include the redundant text, got: {}",
            violation.message
        );
    }
}
