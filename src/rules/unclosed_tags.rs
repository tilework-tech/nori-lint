use std::collections::HashMap;

use crate::diagnostic::RuleViolation;
use crate::rules::Rule;

pub struct UnclosedTagsRule;

impl Rule for UnclosedTagsRule {
    fn name(&self) -> &str {
        "unclosed_tags"
    }

    fn description(&self) -> &str {
        "Checks that all opened XML-style tags have corresponding closing tags"
    }

    fn run(&self, input: &str) -> Option<RuleViolation> {
        let mut open_counts: HashMap<&str, usize> = HashMap::new();
        let mut close_counts: HashMap<&str, usize> = HashMap::new();

        let mut rest = input;
        while let Some(start) = rest.find('<') {
            let after_open = &rest[start + 1..];
            if let Some(end) = after_open.find('>') {
                let tag_content = &after_open[..end];
                if let Some(name) = tag_content.strip_prefix('/') {
                    if !name.is_empty() && !name.contains(' ') {
                        *close_counts.entry(name).or_insert(0) += 1;
                    }
                } else if !tag_content.is_empty()
                    && !tag_content.contains(' ')
                    && !tag_content.starts_with('!')
                    && !tag_content.ends_with('/')
                {
                    *open_counts.entry(tag_content).or_insert(0) += 1;
                }
                rest = &after_open[end + 1..];
            } else {
                break;
            }
        }

        let mut mismatched: Vec<String> = Vec::new();

        for (tag, open) in &open_counts {
            let close = close_counts.get(tag).copied().unwrap_or(0);
            if *open != close {
                mismatched.push(format!("<{tag}>: {} opening, {} closing", open, close));
            }
        }

        for tag in close_counts.keys() {
            if !open_counts.contains_key(tag) {
                mismatched.push(format!("</{tag}> found without opening <{tag}>"));
            }
        }

        mismatched.sort();

        if mismatched.is_empty() {
            None
        } else {
            Some(RuleViolation {
                message: format!("Unclosed or unmatched tags: {}", mismatched.join("; ")),
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
    fn returns_none_when_all_tags_closed() {
        let rule = UnclosedTagsRule;
        let content = "<required>\nContent.\n</required>\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn returns_violation_when_opening_tag_without_closing() {
        let rule = UnclosedTagsRule;
        let content = "<required>\nContent without closing tag.\n";
        let result = rule.run(content);
        assert!(result.is_some(), "should flag unclosed <required> tag");
        let violation = result.unwrap();
        assert!(
            violation.message.contains("required"),
            "message should mention the unclosed tag name, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_violation_when_closing_tag_without_opening() {
        let rule = UnclosedTagsRule;
        let content = "Content.\n</required>\n";
        let result = rule.run(content);
        assert!(result.is_some(), "should flag closing tag without opener");
        let violation = result.unwrap();
        assert!(
            violation.message.contains("required"),
            "message should mention the orphan tag name, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_with_multiple_different_tags_all_closed() {
        let rule = UnclosedTagsRule;
        let content = "<required>\nStuff.\n</required>\n\n<system-reminder>\nMore stuff.\n</system-reminder>\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn returns_violation_when_one_of_multiple_tags_unclosed() {
        let rule = UnclosedTagsRule;
        let content = "<required>\nStuff.\n</required>\n\n<system-reminder>\nNot closed.\n";
        let result = rule.run(content);
        assert!(
            result.is_some(),
            "should flag unclosed <system-reminder> tag"
        );
        let violation = result.unwrap();
        assert!(
            violation.message.contains("system-reminder"),
            "message should mention the unclosed tag, got: {}",
            violation.message
        );
    }

    #[test]
    fn returns_none_when_no_tags_at_all() {
        let rule = UnclosedTagsRule;
        let content = "Just plain text with no XML tags.\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn returns_none_with_multiple_instances_of_same_tag() {
        let rule = UnclosedTagsRule;
        let content = "<required>\nFirst.\n</required>\n\n<required>\nSecond.\n</required>\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn returns_violation_when_counts_dont_match() {
        let rule = UnclosedTagsRule;
        let content = "<required>\nFirst.\n</required>\n\n<required>\nSecond without close.\n";
        let result = rule.run(content);
        assert!(
            result.is_some(),
            "should flag mismatched required tag counts"
        );
    }

    #[test]
    fn returns_none_for_self_closing_tags() {
        let rule = UnclosedTagsRule;
        let content = "Some content with <br/> and <hr/> self-closing tags.\n";
        assert!(rule.run(content).is_none());
    }

    #[test]
    fn has_correct_name() {
        let rule = UnclosedTagsRule;
        assert_eq!(rule.name(), "unclosed_tags");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = UnclosedTagsRule;
        assert!(!rule.description().is_empty());
    }
}
