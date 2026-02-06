use serde::Deserialize;

use crate::diagnostic::RuleViolation;
use crate::rules::llm_rules::{LlmRule, strip_markdown_fences};

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

Respond ONLY with a JSON object in this exact format:
{"has_violations": true/false, "indexes": [{"text": "the first few lines of the offending index", "reason": "why this is a command index"}]}

If there are no violations, respond with:
{"has_violations": false, "indexes": []}"#;

#[derive(Deserialize)]
struct LlmResponse {
    has_violations: bool,
    indexes: Vec<CommandIndex>,
}

#[derive(Deserialize)]
struct CommandIndex {
    text: String,
    #[allow(dead_code)]
    reason: String,
}

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

    fn evaluate(&self, _input: &str, llm_response: &str) -> Option<RuleViolation> {
        let json_str = strip_markdown_fences(llm_response);
        let parsed: LlmResponse = serde_json::from_str(json_str).ok()?;

        if !parsed.has_violations || parsed.indexes.is_empty() {
            return None;
        }

        let details: Vec<String> = parsed
            .indexes
            .iter()
            .map(|idx| format!("\"{}\"", idx.text))
            .collect();

        Some(RuleViolation {
            message: format!(
                "File contains CLI command indexes or reference lists: {}",
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
    fn system_prompt_asks_for_json_response() {
        let rule = CliCommandIndexRule;
        let prompt = rule.system_prompt();
        assert!(
            prompt.contains("has_violations") && prompt.contains("indexes"),
            "system prompt should request JSON with has_violations and indexes fields"
        );
    }

    #[test]
    fn returns_none_when_llm_finds_no_violations() {
        let rule = CliCommandIndexRule;
        let llm_response = r#"{"has_violations": false, "indexes": []}"#;
        let result = rule.evaluate("some skill content", llm_response);
        assert!(result.is_none());
    }

    #[test]
    fn returns_violation_when_llm_finds_command_index() {
        let rule = CliCommandIndexRule;
        let llm_response = r#"{
            "has_violations": true,
            "indexes": [
                {
                    "text": "foo --bar  Use this to do abc\nfoo --baz  Use this to do xyz",
                    "reason": "This is a tabular CLI command index"
                },
                {
                    "text": "Available commands:\nfoo\nfoo bar\nfoo baz",
                    "reason": "This is a bare command list"
                }
            ]
        }"#;
        let result = rule.evaluate("some skill content", llm_response);
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
        let llm_response = r#"{
            "has_violations": true,
            "indexes": [
                {
                    "text": "git status\ngit add\ngit commit\ngit push",
                    "reason": "Bare list of git commands"
                }
            ]
        }"#;
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("git status"),
            "violation message should include the offending text, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_for_malformed_llm_response() {
        let rule = CliCommandIndexRule;
        let llm_response = "this is not json at all";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None for unparseable LLM response rather than panicking"
        );
    }

    #[test]
    fn returns_none_when_has_violations_true_but_indexes_empty() {
        let rule = CliCommandIndexRule;
        let llm_response = r#"{"has_violations": true, "indexes": []}"#;
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should return None when no actual indexes are listed"
        );
    }

    #[test]
    fn handles_markdown_fenced_json_response() {
        let rule = CliCommandIndexRule;
        let llm_response = "```json\n{\"has_violations\": true, \"indexes\": [{\"text\": \"npm install\\nnpm start\\nnpm test\", \"reason\": \"bare command list\"}]}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("npm install"),
            "should parse JSON from within markdown fences, got: {}",
            violation.message
        );
    }

    #[test]
    fn handles_bare_backtick_fenced_response() {
        let rule = CliCommandIndexRule;
        let llm_response = "```\n{\"has_violations\": false, \"indexes\": []}\n```";
        let result = rule.evaluate("content", llm_response);
        assert!(
            result.is_none(),
            "should parse JSON from bare backtick fences"
        );
    }
}
