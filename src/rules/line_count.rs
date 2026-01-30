use crate::diagnostic::RuleViolation;
use crate::rules::Rule;

const MAX_LINES: usize = 150;

pub struct LineCountRule;

impl Rule for LineCountRule {
    fn name(&self) -> &str {
        "line_count"
    }

    fn description(&self) -> &str {
        "Checks that SKILL.md files do not exceed 150 lines"
    }

    fn run(&self, input: &str) -> Option<RuleViolation> {
        let count = input.lines().count();
        if count > MAX_LINES {
            Some(RuleViolation {
                message: format!("File has {count} lines, exceeding the limit of {MAX_LINES}"),
                line: None,
                snippet: None,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_short_file() {
        let rule = LineCountRule;
        let content = "line1\nline2\nline3\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn returns_none_at_exactly_150_lines() {
        let rule = LineCountRule;
        let content: String = (1..=150)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rule.run(&content).is_none());
    }

    #[test]
    fn returns_violation_at_151_lines() {
        let rule = LineCountRule;
        let content: String = (1..=151)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let result = rule.run(&content);
        assert!(result.is_some());
        let violation = result.unwrap();
        assert!(
            violation.message.contains("151"),
            "error should mention actual line count, got: {}",
            violation.message
        );
        assert!(
            violation.message.contains("150"),
            "error should mention the limit, got: {}",
            violation.message
        );
        assert!(
            violation.line.is_none(),
            "line_count is a file-level rule, line should be None"
        );
        assert!(
            violation.snippet.is_none(),
            "line_count is a file-level rule, snippet should be None"
        );
    }

    #[test]
    fn has_correct_name() {
        let rule = LineCountRule;
        assert_eq!(rule.name(), "line_count");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = LineCountRule;
        assert!(!rule.description().is_empty());
    }
}
