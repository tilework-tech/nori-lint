use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, strip_markdown_fences};

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

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "instructions": [{"text": "the exact offending instruction", "reason": "why this is obvious"}]}

If there are no violations, respond with:
{"has_violations": false, "instructions": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    instructions: Vec<Instruction>,
}

#[derive(Deserialize)]
struct Instruction {
    text: String,
    #[allow(dead_code)]
    reason: String,
}

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

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.instructions.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .instructions
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
    fn system_prompt_asks_for_json_response() {
        let rule = ObviousInstructionsRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("instructions"),
            "system prompt should request JSON with has_violations and instructions fields"
        );
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = ObviousInstructionsRule;
        let llm_response = r#"{"has_violations": false, "instructions": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_obvious_instructions_found() {
        let rule = ObviousInstructionsRule;
        let llm_response = r#"{
            "has_violations": true,
            "instructions": [
                {
                    "text": "read code changes carefully",
                    "reason": "Any LLM will already read input carefully"
                },
                {
                    "text": "keep code modular where possible",
                    "reason": "Basic software practice any LLM knows"
                }
            ]
        }"#;
        let result = rule.evaluate("content with obvious instructions", llm_response);
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
        let llm_response = r#"{"has_violations": true, "instructions": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual instructions are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json() {
        let rule = ObviousInstructionsRule;
        let llm_response = "```json\n{\"has_violations\": true, \"instructions\": [{\"text\": \"follow best practices\", \"reason\": \"obvious\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("follow best practices"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_malformed_json() {
        let rule = ObviousInstructionsRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }
}
