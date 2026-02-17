use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

const SYSTEM_PROMPT: &str = r#"You are reviewing a SKILL.md file. These files are instructions written by a human to configure an AI assistant's behavior. The audience is the AI assistant itself.

Find passages where the author refers to themselves in the third person as "the user" instead of using first person ("me", "my", "I"). Since the human is writing instructions for the AI, they should address themselves in first person.

<bad_example>
"Ask the user to answer the question"
"Tell the user about the error"
"Wait for the user's response"
"Show the user the results"
"If the user wants to proceed"
</bad_example>

<good_example>
"Ask me to answer the question"
"Tell me about the error"
"Wait for my response"
"Show me the results"
"If I want to proceed"
</good_example>

Only flag cases where "the user" clearly refers to the author of the skill file. Do NOT flag:
- References to end users of a product being built (e.g., "validate the user's input" in a web app context)
- Quoted examples or code snippets
- References to third-party users

For each violation found, report the offending passage as "text" and the corrected first-person version as "reason"."#;

pub struct FirstPersonRule;

impl LlmRule for FirstPersonRule {
    fn name(&self) -> &str {
        "first_person"
    }

    fn description(&self) -> &str {
        "Checks that skill files use first person ('me', 'I') instead of 'the user'"
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
            .map(|v| format!("\"{}\" -> \"{}\"", v.text, v.reason))
            .collect();
        Some(RuleViolation {
            message: format!(
                "File refers to the skill author as 'the user' instead of first person: {}",
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
        let rule = FirstPersonRule;
        assert_eq!(rule.name(), "first_person");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = FirstPersonRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = FirstPersonRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = FirstPersonRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_third_person_references() {
        let rule = FirstPersonRule;
        let result = rule.evaluate(
            "Ask the user to answer the question",
            &[LlmViolation {
                text: "Ask the user to answer the question".into(),
                reason: "Ask me to answer the question".into(),
            }],
        );
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation
                .message
                .contains("Ask the user to answer the question"),
            "violation message should include the offending text, got: {}",
            violation.message
        );
        assert!(
            violation.message.contains("Ask me to answer the question"),
            "violation message should include the suggested replacement, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_violation_with_multiple_third_person_references() {
        let rule = FirstPersonRule;
        let result = rule.evaluate(
            "content",
            &[
                LlmViolation {
                    text: "Tell the user about the error".into(),
                    reason: "Tell me about the error".into(),
                },
                LlmViolation {
                    text: "Wait for the user's response".into(),
                    reason: "Wait for my response".into(),
                },
            ],
        );
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Tell the user about the error"),
            "should include first offending text, got: {}",
            violation.message
        );
        assert!(
            violation.message.contains("Wait for the user's response"),
            "should include second offending text, got: {}",
            violation.message
        );
    }
}
