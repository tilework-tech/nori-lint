use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::LlmRule;

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

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "explanations": [{"text": "the exact offending passage", "reason": "why this is redundant"}]}

If there are no violations, respond with:
{"has_violations": false, "explanations": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    explanations: Vec<Explanation>,
}

#[derive(Deserialize)]
struct Explanation {
    text: String,
    #[allow(dead_code)]
    reason: String,
}

fn strip_markdown_fences(s: &str) -> &str {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let inner = if let Some(after_lang) = rest.split_once('\n') {
            after_lang.1
        } else {
            rest
        };
        inner.strip_suffix("```").unwrap_or(inner).trim()
    } else {
        trimmed
    }
}

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

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.explanations.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .explanations
            .iter()
            .map(|e| format!("\"{}\"", e.text))
            .collect();

        Some(RuleViolation {
            message: format!(
                "File contains explanations of concepts an LLM already knows: {}",
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
    fn system_prompt_asks_for_json_response() {
        let rule = RedundantExplanationRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("explanations"),
            "system prompt should request JSON with has_violations and explanations fields"
        );
    }

    #[test]
    fn returns_none_when_llm_finds_no_violations() {
        let rule = RedundantExplanationRule;
        let llm_response = r#"{"has_violations": false, "explanations": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_redundant_explanations() {
        let rule = RedundantExplanationRule;
        let llm_response = r#"{
            "has_violations": true,
            "explanations": [
                {
                    "text": "GCP stands for Google Cloud Platform",
                    "reason": "LLMs are trained on extensive documentation about GCP"
                },
                {
                    "text": "JSON is a lightweight data interchange format",
                    "reason": "JSON is fundamental knowledge for any LLM"
                }
            ]
        }"#;
        let result = rule.evaluate("some skill content with explanations", llm_response);
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
        let llm_response = r#"{
            "has_violations": true,
            "explanations": [
                {
                    "text": "REST stands for Representational State Transfer",
                    "reason": "Well-known concept"
                }
            ]
        }"#;
        let result = rule.evaluate("content", llm_response);
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

    #[test]
    fn returns_none_for_malformed_llm_response() {
        let rule = RedundantExplanationRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn returns_none_when_has_violations_is_true_but_explanations_empty() {
        let rule = RedundantExplanationRule;
        let llm_response = r#"{"has_violations": true, "explanations": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual explanations are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json_response() {
        let rule = RedundantExplanationRule;
        let llm_response = "```json\n{\"has_violations\": true, \"explanations\": [{\"text\": \"Git is a version control system\", \"reason\": \"well known\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation
                .message
                .contains("Git is a version control system"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn handles_bare_backtick_fenced_response() {
        let rule = RedundantExplanationRule;
        let llm_response = "```\n{\"has_violations\": false, \"explanations\": []}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should parse JSON from bare backtick fences"
        );
    }
}
