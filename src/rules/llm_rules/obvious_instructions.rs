use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

const SYSTEM_PROMPT: &str = r#"Find instructions in the provided SKILL.md file that are so generic and obvious that any LLM would already follow them without being told. These are instructions that waste tokens by stating things the LLM already knows to do.

<bad_example>
"Read code changes carefully"
"Understand the scope of modifications"
"Keep code modular where possible"
"Follow best practices"
"Ensure code quality"
"Write clean code"
"Maintain readability"
"Use meaningful variable names"
"Add comments where necessary"
"Handle errors appropriately"
"Test your changes"
"Review your work"
"Pay attention to detail"
"Consider edge cases"
"Think about performance"
"Ensure proper documentation"
"Follow coding standards"
"Keep functions small"
"Avoid code duplication"
"Understand the codebase"
</bad_example>

These should be removed because they add no value. An LLM already knows to do all of these things.

Instructions that ARE valuable and should NOT be flagged:

<good_example>
"NEVER push to main without permission"
"Use the tokio runtime with current_thread flavor"
"Always run cargo clippy before committing"
"Prefix all API routes with /v2/"
"Log errors to stderr, not stdout"
</good_example>

These are valuable because they are specific, project-contextual, or override default LLM behavior.

The key distinction: if the instruction could appear in ANY project's guidelines and adds nothing project-specific, it is obvious. If it constrains behavior in a specific, non-default way, it is valuable.

For each violation found, report the obvious instruction as "text" and explain why it is obvious as "reason"."#;

pub struct ObviousInstructionsRule;

impl LlmRule for ObviousInstructionsRule {
    fn name(&self) -> &str {
        "obvious_instructions"
    }

    fn description(&self) -> &str {
        "Checks that skill files do not contain generic instructions any LLM would already follow"
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
            .map(|i| format!("\"{}\"", i.text))
            .collect();
        Some(RuleViolation {
            message: format!(
                "File contains obvious instructions any LLM would already follow: {}",
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
        let rule = ObviousInstructionsRule;
        assert_eq!(rule.name(), "obvious_instructions");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = ObviousInstructionsRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = ObviousInstructionsRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = ObviousInstructionsRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_obvious_instructions_found() {
        let rule = ObviousInstructionsRule;
        let violations = &[
            LlmViolation {
                text: "read code changes carefully".into(),
                reason: "Any LLM will already read input carefully".into(),
            },
            LlmViolation {
                text: "keep code modular where possible".into(),
                reason: "Basic software practice any LLM knows".into(),
            },
        ];
        let result = rule.evaluate("content with obvious instructions", violations);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("read code changes carefully"),
            "violation message should include the flagged text, got: {}",
            violation.message
        );
        assert!(
            violation
                .message
                .contains("keep code modular where possible"),
            "violation message should include all flagged texts, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_empty_instructions_even_if_flagged() {
        let rule = ObviousInstructionsRule;
        let result = rule.evaluate("content", &[]);
        assert!(
            result.is_none(),
            "should return None when no actual instructions are listed"
        );
    }
}
