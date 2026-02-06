use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{strip_markdown_fences, LlmRule};

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

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "violations": [{"original": "the exact offending passage", "suggested": "the corrected first-person version"}]}

If there are no violations, respond with:
{"has_violations": false, "violations": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    violations: Vec<Violation>,
}

#[derive(Deserialize)]
struct Violation {
    original: String,
    suggested: String,
}

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

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.violations.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .violations
            .iter()
            .map(|v| format!("\"{}\" -> \"{}\"", v.original, v.suggested))
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
    fn system_prompt_asks_for_json_response() {
        let rule = FirstPersonRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("violations"),
            "system prompt should request JSON with has_violations and violations fields"
        );
    }

    #[test]
    fn returns_none_when_llm_finds_no_violations() {
        let rule = FirstPersonRule;
        let llm_response = r#"{"has_violations": false, "violations": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_third_person_references() {
        let rule = FirstPersonRule;
        let llm_response = r#"{
            "has_violations": true,
            "violations": [
                {
                    "original": "Ask the user to answer the question",
                    "suggested": "Ask me to answer the question"
                }
            ]
        }"#;
        let result = rule.evaluate("Ask the user to answer the question", llm_response);
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
        let llm_response = r#"{
            "has_violations": true,
            "violations": [
                {
                    "original": "Tell the user about the error",
                    "suggested": "Tell me about the error"
                },
                {
                    "original": "Wait for the user's response",
                    "suggested": "Wait for my response"
                }
            ]
        }"#;
        let result = rule.evaluate("content", llm_response);
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

    #[test]
    fn returns_none_for_malformed_llm_response() {
        let rule = FirstPersonRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn returns_none_when_has_violations_is_true_but_violations_empty() {
        let rule = FirstPersonRule;
        let llm_response = r#"{"has_violations": true, "violations": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual violations are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json_response() {
        let rule = FirstPersonRule;
        let llm_response = "```json\n{\"has_violations\": true, \"violations\": [{\"original\": \"Ask the user for input\", \"suggested\": \"Ask me for input\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("Ask the user for input"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn handles_bare_backtick_fenced_response() {
        let rule = FirstPersonRule;
        let llm_response = "```\n{\"has_violations\": false, \"violations\": []}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should parse JSON from bare backtick fences"
        );
    }

    #[test]
    fn returns_none_when_has_violations_is_false_but_violations_non_empty() {
        let rule = FirstPersonRule;
        let llm_response = r#"{
            "has_violations": false,
            "violations": [
                {
                    "original": "Tell the user about the error",
                    "suggested": "Tell me about the error"
                }
            ]
        }"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should trust has_violations flag over array contents"
        );
    }
}
