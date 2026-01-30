use crate::diagnostic::RuleViolation;
use crate::rules::Rule;

pub struct RequiredTagsRule;

impl Rule for RequiredTagsRule {
    fn name(&self) -> &str {
        "required_tags"
    }

    fn description(&self) -> &str {
        "Checks that SKILL.md files contain at least one <required> block"
    }

    fn run(&self, input: &str) -> Option<RuleViolation> {
        let has_open = input.contains("<required>");
        let has_close = input.contains("</required>");

        if has_open && has_close {
            None
        } else {
            Some(RuleViolation {
                message: "File is missing a <required> block".to_string(),
                line: None,
                snippet: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_when_required_tags_present() {
        let rule = RequiredTagsRule;
        let content = "---\nname: Test\n---\n\n<required>\nDo things.\n</required>\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn returns_violation_when_no_required_tags() {
        let rule = RequiredTagsRule;
        let content = "---\nname: Test\n---\n\nSome content without required tags.\n";
        let result = rule.run(content);
        assert!(result.is_some(), "should flag missing <required> tags");
        let violation = result.unwrap();
        assert!(
            violation.message.contains("required"),
            "message should mention required tags, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_when_multiple_required_pairs() {
        let rule = RequiredTagsRule;
        let content =
            "<required>\nFirst block.\n</required>\n\n<required>\nSecond block.\n</required>\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn has_correct_name() {
        let rule = RequiredTagsRule;
        assert_eq!(rule.name(), "required_tags");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = RequiredTagsRule;
        assert!(!rule.description().is_empty());
    }
}
