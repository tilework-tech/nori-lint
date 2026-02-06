use crate::diagnostic::RuleViolation;
use crate::rules::Rule;

fn strip_inline_code(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut chars = line.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '`' {
            let mut found_close = false;
            for (j, c2) in &mut chars {
                if c2 == '`' {
                    for _ in i..=j {
                        result.push(' ');
                    }
                    found_close = true;
                    break;
                }
            }
            if !found_close {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn is_example_tag_open(trimmed: &str) -> bool {
    trimmed == "<good-example>"
        || trimmed == "<bad-example>"
        || trimmed == "<good_example>"
        || trimmed == "<bad_example>"
}

fn is_example_tag_close(trimmed: &str) -> bool {
    trimmed == "</good-example>"
        || trimmed == "</bad-example>"
        || trimmed == "</good_example>"
        || trimmed == "</bad_example>"
}

pub struct MarkdownLinksRule;

impl Rule for MarkdownLinksRule {
    fn name(&self) -> &str {
        "markdown_links"
    }

    fn description(&self) -> &str {
        "Checks that SKILL.md files do not use markdown link syntax; URLs should be bare links"
    }

    fn run(&self, input: &str) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let mut in_code_block = false;
        let mut in_example_block = false;

        for (line_idx, line) in input.lines().enumerate() {
            let line_num = (line_idx + 1) as u32;
            let trimmed = line.trim();

            // Example block boundaries must be checked before code fences,
            // because example blocks may contain code fences that should not
            // toggle the in_code_block state.
            if is_example_tag_open(trimmed) {
                in_example_block = true;
                continue;
            }
            if is_example_tag_close(trimmed) {
                in_example_block = false;
                continue;
            }
            if in_example_block {
                continue;
            }

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Skip reference-style link definitions: [label]: url
            if trimmed.starts_with('[')
                && let Some(bracket_end) = trimmed.find(']')
                && trimmed[bracket_end + 1..].starts_with(": ")
            {
                continue;
            }

            let check_line = strip_inline_code(line);
            self.find_markdown_links(&check_line, line, line_num, &mut violations);
        }

        violations
    }
}

impl MarkdownLinksRule {
    fn find_markdown_links(
        &self,
        check_line: &str,
        original_line: &str,
        line_num: u32,
        violations: &mut Vec<RuleViolation>,
    ) {
        let bytes = check_line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'[' {
                // Check it's not an image: ![
                if i > 0 && bytes[i - 1] == b'!' {
                    i += 1;
                    continue;
                }

                // Find closing ]
                if let Some(close_bracket) = check_line[i + 1..].find(']') {
                    let close_pos = i + 1 + close_bracket;
                    let link_text = &check_line[i + 1..close_pos];

                    // Must have non-empty link text
                    if link_text.is_empty() {
                        i = close_pos + 1;
                        continue;
                    }

                    // Must be immediately followed by (
                    if close_pos + 1 < bytes.len() && bytes[close_pos + 1] == b'(' {
                        // Find matching )
                        if let Some(close_paren) = check_line[close_pos + 2..].find(')') {
                            let paren_end = close_pos + 2 + close_paren;
                            let matched = &check_line[i..paren_end + 1];
                            violations.push(RuleViolation {
                                message: format!(
                                    "Markdown link syntax found: {}. Use bare URLs instead",
                                    matched
                                ),
                                line: Some(line_num),
                                snippet: Some(original_line.to_string()),
                            });
                            i = paren_end + 1;
                            continue;
                        }
                    }
                    i = close_pos + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_correct_name() {
        let rule = MarkdownLinksRule;
        assert_eq!(rule.name(), "markdown_links");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = MarkdownLinksRule;
        assert!(!rule.description().is_empty());
    }

    #[test]
    fn flags_standard_markdown_link() {
        let rule = MarkdownLinksRule;
        let content = "Check out [this site](https://example.com) for more.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(1));
        assert!(
            result[0]
                .message
                .contains("[this site](https://example.com)")
        );
    }

    #[test]
    fn flags_markdown_link_with_title() {
        let rule = MarkdownLinksRule;
        let content = "See [docs](https://example.com \"The Docs\") for details.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(1));
    }

    #[test]
    fn does_not_flag_bare_url() {
        let rule = MarkdownLinksRule;
        let content = "Visit https://example.com for more.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_inside_fenced_code_block() {
        let rule = MarkdownLinksRule;
        let content = "Normal text.\n```\nSee [link](https://example.com)\n```\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_inside_fenced_code_block_with_language() {
        let rule = MarkdownLinksRule;
        let content =
            "Normal text.\n```markdown\nSee [link](https://example.com)\n```\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_inside_inline_code() {
        let rule = MarkdownLinksRule;
        let content = "Use `[link](https://example.com)` syntax for links.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_image_syntax() {
        let rule = MarkdownLinksRule;
        let content = "Here is an image: ![alt text](https://example.com/img.png)\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn reports_multiple_violations_on_different_lines() {
        let rule = MarkdownLinksRule;
        let content = "See [foo](https://foo.com) here.\nAnd [bar](https://bar.com) there.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].line, Some(1));
        assert_eq!(result[1].line, Some(2));
    }

    #[test]
    fn does_not_flag_reference_style_definitions() {
        let rule = MarkdownLinksRule;
        let content = "[label]: https://example.com\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_links_inside_good_example_tags() {
        let rule = MarkdownLinksRule;
        let content = "Some text.\n<good-example>\nSee [link](https://example.com)\n</good-example>\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_links_inside_bad_example_tags() {
        let rule = MarkdownLinksRule;
        let content = "Some text.\n<bad-example>\nSee [link](https://example.com)\n</bad-example>\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_links_inside_underscore_example_tags() {
        let rule = MarkdownLinksRule;
        let content = "Some text.\n<good_example>\nSee [link](https://example.com)\n</good_example>\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn flags_links_outside_example_tags() {
        let rule = MarkdownLinksRule;
        let content = "<good-example>\nSafe [link](https://example.com)\n</good-example>\nUnsafe [link](https://example.com) here.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(4));
    }

    #[test]
    fn reports_multiple_violations_on_same_line() {
        let rule = MarkdownLinksRule;
        let content = "See [foo](https://foo.com) and [bar](https://bar.com) here.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].line, Some(1));
        assert_eq!(result[1].line, Some(1));
    }

    #[test]
    fn does_not_leak_code_block_state_from_example_block() {
        let rule = MarkdownLinksRule;
        let content = "<good-example>\n```\ncode\n</good-example>\n[link](https://example.com)\n";
        let result = rule.run(content);
        assert_eq!(
            result.len(),
            1,
            "link outside example block should be flagged even when example block contains a code fence"
        );
    }

    #[test]
    fn returns_empty_for_clean_content() {
        let rule = MarkdownLinksRule;
        let content = "---\nname: Test\n---\n\n<required>\nPlain text with https://example.com as bare link.\n</required>\n";
        assert!(rule.run(content).is_empty());
    }
}
