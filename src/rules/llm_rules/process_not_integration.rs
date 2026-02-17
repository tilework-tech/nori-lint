use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

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

Only set has_violations to true if you have HIGH confidence that this file is an integration/reference manual rather than a process. If confidence is medium or low, set has_violations to false.

For each piece of evidence, report the passage that indicates integration style as "text" and explain why it is integration-style content as "reason"."#;

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
    use crate::rules::llm_rules::LlmViolation;

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
    fn returns_none_when_skill_is_a_process() {
        let rule = ProcessNotIntegrationRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(
            result.is_none(),
            "should return None when skill is identified as a process"
        );
    }

    #[test]
    fn returns_violation_when_skill_is_an_integration_with_high_confidence() {
        let rule = ProcessNotIntegrationRule;
        let violations = &[
            LlmViolation {
                text: "--name (required): Clear, searchable title".into(),
                reason: "Parameter reference listing typical of integration docs".into(),
            },
            LlmViolation {
                text: "Returns confirmation with artifact ID".into(),
                reason: "Declarative output description rather than a process step".into(),
            },
        ];
        let result = rule.evaluate("some integration-style content", violations);
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
    fn returns_none_when_is_integration_true_but_evidence_empty() {
        let rule = ProcessNotIntegrationRule;
        let result = rule.evaluate("content", &[]);
        assert!(
            result.is_none(),
            "should return None when no actual evidence is listed"
        );
    }
}
