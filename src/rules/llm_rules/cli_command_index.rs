use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, LlmViolation};

const SYSTEM_PROMPT: &str = r#"Find sections in the provided file that contain a CLI command index or reference list. These are sections where the author lists out CLI commands, subcommands, or flags in an index-like format rather than providing meaningful context about when and why to use them.

<bad_example>
foo --bar  Use this to do abc
foo --baz  Use this to do xyz
foo --qux  Use this to do 123
</bad_example>

<bad_example>
Available commands:
foo
foo bar
foo baz
foo qux
</bad_example>

<bad_example>
| Command | Description |
|---------|-------------|
| foo bar | Does thing A |
| foo baz | Does thing B |
</bad_example>

These command indexes waste tokens. An LLM already has knowledge of CLI tools from its training data, and a flat list of commands with brief descriptions provides no useful context about *when* or *why* to use them.

Do NOT flag:
- A single CLI example used to illustrate a specific point
- A command shown as part of a step-by-step workflow (e.g., "Run `foo --bar` to enable logging, then check the output")
- Code blocks showing example usage with meaningful surrounding context

For each violation found, report the offending command index text as "text" and explain why it wastes tokens as "reason"."#;

pub struct CliCommandIndexRule;

impl LlmRule for CliCommandIndexRule {
    fn name(&self) -> &str {
        "cli_command_index"
    }

    fn description(&self) -> &str {
        "Checks that skill files do not contain CLI command indexes or reference lists"
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
                "File contains CLI command indexes that waste tokens: {}",
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
        let rule = CliCommandIndexRule;
        assert_eq!(rule.name(), "cli_command_index");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = CliCommandIndexRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn system_prompt_is_non_empty() {
        let rule = CliCommandIndexRule;
        assert!(!rule.system_prompt().is_empty());
    }

    #[test]
    fn returns_none_when_no_violations() {
        let rule = CliCommandIndexRule;
        let result = rule.evaluate("some skill content", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_command_index() {
        let rule = CliCommandIndexRule;
        let result = rule.evaluate(
            "some skill content",
            &[
                LlmViolation {
                    text: "foo --bar  Use this to do abc\nfoo --baz  Use this to do xyz".into(),
                    reason: "This is a tabular CLI command index".into(),
                },
                LlmViolation {
                    text: "Available commands:\nfoo\nfoo bar\nfoo baz".into(),
                    reason: "This is a bare command list".into(),
                },
            ],
        );
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("foo --bar"),
            "violation message should include the offending text, got: {}",
            violation.message
        );
        assert!(
            violation.message.contains("Available commands"),
            "violation message should include all offending indexes, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_violation_with_single_index() {
        let rule = CliCommandIndexRule;
        let result = rule.evaluate(
            "content",
            &[LlmViolation {
                text: "git status\ngit add\ngit commit\ngit push".into(),
                reason: "Bare list of git commands".into(),
            }],
        );
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("git status"),
            "violation message should include the offending text, got: {}",
            violation.message
        );
    }
}
