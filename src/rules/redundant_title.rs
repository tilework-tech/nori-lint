use crate::diagnostic::RuleViolation;
use crate::rules::Rule;

pub struct RedundantTitleRule;

impl Rule for RedundantTitleRule {
    fn name(&self) -> &str {
        "redundant_title"
    }

    fn description(&self) -> &str {
        "Checks that SKILL.md files do not start with a title heading"
    }

    fn run(&self, input: &str) -> Vec<RuleViolation> {
        let lines: Vec<&str> = input.lines().collect();
        let start = skip_frontmatter(&lines);

        // Find the first non-empty line after frontmatter
        let first_content = lines[start..]
            .iter()
            .enumerate()
            .find(|(_, line)| !line.trim().is_empty());

        let Some((offset, line)) = first_content else {
            return vec![];
        };
        let line_number = (start + offset + 1) as u32;

        // Check ATX heading (# ...)
        if line.starts_with('#') {
            return vec![RuleViolation {
                message: "File starts with a title heading instead of useful content".to_string(),
                line: Some(line_number),
                snippet: Some(line.trim().to_string()),
            }];
        }

        // Check setext heading: current line is text, next line is all = or all -
        let next_idx = start + offset + 1;
        if next_idx < lines.len() {
            let next_line = lines[next_idx].trim();
            if !next_line.is_empty() && is_setext_underline(next_line) {
                return vec![RuleViolation {
                    message: "File starts with a title heading instead of useful content"
                        .to_string(),
                    line: Some(line_number),
                    snippet: Some(line.trim().to_string()),
                }];
            }
        }

        vec![]
    }
}

fn skip_frontmatter(lines: &[&str]) -> usize {
    if lines.is_empty() || lines[0].trim() != "---" {
        return 0;
    }
    // Find closing ---
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            return i + 1;
        }
    }
    // Unclosed frontmatter — treat entire file as frontmatter
    lines.len()
}

fn is_setext_underline(line: &str) -> bool {
    (!line.is_empty()) && (line.chars().all(|c| c == '=') || line.chars().all(|c| c == '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_violation_for_atx_heading_at_start() {
        let rule = RedundantTitleRule;
        let content = "# The Foo Document\n\nThis document describes foo.\n";
        let result = rule.run(content);
        assert_eq!(
            result.len(),
            1,
            "should flag a file starting with a heading"
        );
        assert_eq!(result[0].line, Some(1), "should report a line number");
        assert!(
            result[0]
                .snippet
                .as_ref()
                .unwrap()
                .contains("# The Foo Document"),
            "should include the heading as snippet"
        );
    }

    #[test]
    fn returns_empty_for_body_text_at_start() {
        let rule = RedundantTitleRule;
        let content = "This document describes foo, a useful tool.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn returns_violation_for_heading_after_frontmatter() {
        let rule = RedundantTitleRule;
        let content = "---\nname: Foo\ndescription: A foo skill\n---\n\n# The Foo Document\n\nThis document describes foo.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1, "should flag heading after frontmatter");
        assert_eq!(result[0].line, Some(6), "heading is on line 6");
        assert!(
            result[0]
                .snippet
                .as_ref()
                .unwrap()
                .contains("# The Foo Document")
        );
    }

    #[test]
    fn returns_empty_for_body_text_after_frontmatter() {
        let rule = RedundantTitleRule;
        let content =
            "---\nname: Foo\ndescription: A foo skill\n---\n\nThis document describes foo.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn returns_empty_for_empty_file() {
        let rule = RedundantTitleRule;
        assert!(rule.run("").is_empty());
    }

    #[test]
    fn returns_empty_for_whitespace_only_file() {
        let rule = RedundantTitleRule;
        assert!(rule.run("  \n\n  \n").is_empty());
    }

    #[test]
    fn returns_violation_for_h2_heading() {
        let rule = RedundantTitleRule;
        let content = "## Section Title\n\nSome content.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1, "should flag any heading level");
        assert_eq!(result[0].line, Some(1));
        assert!(
            result[0]
                .snippet
                .as_ref()
                .unwrap()
                .contains("## Section Title")
        );
    }

    #[test]
    fn returns_violation_for_h3_heading() {
        let rule = RedundantTitleRule;
        let content = "### Deep Title\n\nSome content.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1, "should flag any heading level");
    }

    #[test]
    fn returns_violation_for_setext_h1() {
        let rule = RedundantTitleRule;
        let content = "The Foo Document\n================\n\nThis describes foo.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1, "should flag setext h1 heading");
        assert_eq!(result[0].line, Some(1));
        assert!(
            result[0]
                .snippet
                .as_ref()
                .unwrap()
                .contains("The Foo Document")
        );
    }

    #[test]
    fn returns_violation_for_setext_h2() {
        let rule = RedundantTitleRule;
        let content = "The Foo Document\n----------------\n\nThis describes foo.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1, "should flag setext h2 heading");
        assert_eq!(result[0].line, Some(1));
        assert!(
            result[0]
                .snippet
                .as_ref()
                .unwrap()
                .contains("The Foo Document")
        );
    }

    #[test]
    fn returns_violation_for_setext_heading_after_frontmatter() {
        let rule = RedundantTitleRule;
        let content = "---\nname: Foo\ndescription: A foo skill\n---\n\nThe Foo Document\n================\n\nThis describes foo.\n";
        let result = rule.run(content);
        assert_eq!(
            result.len(),
            1,
            "should flag setext heading after frontmatter"
        );
        assert_eq!(result[0].line, Some(6));
        assert!(
            result[0]
                .snippet
                .as_ref()
                .unwrap()
                .contains("The Foo Document")
        );
    }

    #[test]
    fn does_not_confuse_frontmatter_delimiter_with_setext() {
        let rule = RedundantTitleRule;
        let content = "---\nname: Foo\n---\n\nThis is body text.\n";
        assert!(
            rule.run(content).is_empty(),
            "frontmatter delimiter should not be treated as setext underline"
        );
    }

    #[test]
    fn returns_empty_for_no_content_after_frontmatter() {
        let rule = RedundantTitleRule;
        let content = "---\nname: Foo\n---\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn returns_empty_when_heading_appears_later_in_body() {
        let rule = RedundantTitleRule;
        let content = "---\nname: Foo\ndescription: A foo skill\n---\n\nThis is body text.\n\n# Section Heading\n\nMore content.\n";
        assert!(
            rule.run(content).is_empty(),
            "headings after the first content line should not trigger the rule"
        );
    }

    #[test]
    fn has_correct_name() {
        let rule = RedundantTitleRule;
        assert_eq!(rule.name(), "redundant_title");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = RedundantTitleRule;
        assert!(!rule.description().is_empty());
    }
}
