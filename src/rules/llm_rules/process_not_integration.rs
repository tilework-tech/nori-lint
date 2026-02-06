use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, strip_markdown_fences};

const SYSTEM_PROMPT: &str = r#"Determine whether the provided SKILL.md file is an "integration" (a reference manual) or a "process" (a step-by-step workflow).

A **process skill** defines a clear workflow the agent must follow. It has:
- Numbered checklists of sequential steps inside `<required>` blocks
- Instructions to add steps to a TodoWrite list
- Imperative verbs as step openers: "Write", "Verify", "Run", "Confirm", "Check", "Push"
- Named phases with explicit ordering ("Phase 1", "Phase 2", or "You MUST complete each phase before proceeding")
- Conditional branching ("If tests fail: ... If tests pass: ...")
- Loop constructs ("Follow these steps in a loop until...")
- Stop conditions and red flags

An **integration skill** is essentially a reference manual documenting what a tool can do. It has:
- CLI command templates with parameter documentation (e.g., `--name (required): Clear, searchable title`)
- API endpoint listings or mode/capability catalogs
- Platform comparison tables
- Output format descriptions ("Returns confirmation with artifact ID")
- Declarative tone ("Supports...", "Available...", "Returns...")
- No `<required>` block with a numbered checklist

A skill can be a hybrid — some process content mixed with reference material. Only flag a skill as an integration if the **majority** of its content is reference/catalog material rather than process steps.

Respond ONLY with a JSON object in this exact format:
{"is_integration": true/false, "confidence": "high"/"medium"/"low", "evidence": [{"text": "the exact passage that indicates integration style", "reason": "why this is integration-style content"}]}

If the skill is a process (not an integration), respond with:
{"is_integration": false, "confidence": "high", "evidence": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    is_integration: bool,
    confidence: String,
    evidence: Vec<Evidence>,
}

#[derive(Deserialize)]
struct Evidence {
    text: String,
    #[allow(dead_code)]
    reason: String,
}

pub struct ProcessNotIntegrationRule;

impl LlmRule for ProcessNotIntegrationRule {
    fn name(&self) -> &str {
        "process_not_integration"
    }

    fn description(&self) -> &str {
        "Checks that skill files define a process rather than just documenting a tool integration"
    }

    fn system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.is_integration || parsed.confidence != "high" || parsed.evidence.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .evidence
            .iter()
            .map(|e| format!("\"{}\"", e.text))
            .collect();

        Some(RuleViolation {
            message: format!(
                "Skill file reads as a tool integration/reference manual rather than a process: {}",
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
        let rule = ProcessNotIntegrationRule;
        assert_eq!(rule.name(), "process_not_integration");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = ProcessNotIntegrationRule;
        assert!(
            !rule.description().is_empty(),
            "description should not be empty"
        );
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = ProcessNotIntegrationRule;
        assert!(
            !rule.system_prompt().is_empty(),
            "system prompt should not be empty"
        );
    }

    #[test]
    fn system_prompt_requests_expected_json_format() {
        let rule = ProcessNotIntegrationRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("is_integration") && prompt.contains("evidence"),
            "system prompt should request JSON with is_integration and evidence fields"
        );
    }

    #[test]
    fn returns_none_when_skill_is_a_process() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = r#"{"is_integration": false, "confidence": "high", "evidence": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(
            result.is_none(),
            "should return None when skill is identified as a process"
        );
    }

    #[test]
    fn returns_violation_when_skill_is_an_integration_with_high_confidence() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = r#"{
            "is_integration": true,
            "confidence": "high",
            "evidence": [
                {
                    "text": "--name (required): Clear, searchable title",
                    "reason": "Parameter reference listing typical of integration docs"
                },
                {
                    "text": "Returns confirmation with artifact ID",
                    "reason": "Declarative output description rather than a process step"
                }
            ]
        }"#;
        let result = rule.evaluate("some integration-style content", llm_response);
        assert!(
            result.is_some(),
            "should return a violation for high-confidence integration detection"
        );
        let violation = result.unwrap();
        assert!(
            violation
                .message
                .contains("--name (required): Clear, searchable title"),
            "violation message should include the evidence text, got: {}",
            violation.message
        );
        assert!(
            violation
                .message
                .contains("Returns confirmation with artifact ID"),
            "violation message should include all evidence texts, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_when_confidence_is_medium() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = r#"{
            "is_integration": true,
            "confidence": "medium",
            "evidence": [
                {
                    "text": "some ambiguous content",
                    "reason": "could be either"
                }
            ]
        }"#;
        let result = rule.evaluate("hybrid skill content", llm_response);
        assert!(
            result.is_none(),
            "should return None when confidence is not high"
        );
    }

    #[test]
    fn returns_none_when_confidence_is_low() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = r#"{
            "is_integration": true,
            "confidence": "low",
            "evidence": [
                {
                    "text": "minor reference material",
                    "reason": "barely qualifies"
                }
            ]
        }"#;
        let result = rule.evaluate("mostly process skill", llm_response);
        assert!(
            result.is_none(),
            "should return None when confidence is low"
        );
    }

    #[test]
    fn returns_none_for_malformed_llm_response() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn returns_none_when_is_integration_true_but_evidence_empty() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = r#"{"is_integration": true, "confidence": "high", "evidence": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual evidence is listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json_response() {
        let rule = ProcessNotIntegrationRule;
        let llm_response = "```json\n{\"is_integration\": true, \"confidence\": \"high\", \"evidence\": [{\"text\": \"--verbose flag enables detailed output\", \"reason\": \"CLI flag reference\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_some(),
            "should parse JSON from within markdown fences"
        );
        let violation = result.unwrap();
        assert!(
            violation
                .message
                .contains("--verbose flag enables detailed output"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn handles_bare_backtick_fenced_response() {
        let rule = ProcessNotIntegrationRule;
        let llm_response =
            "```\n{\"is_integration\": false, \"confidence\": \"high\", \"evidence\": []}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should parse JSON from bare backtick fences"
        );
    }
}
