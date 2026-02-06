use crate::diagnostic::RuleViolation;
use crate::rules::Rule;

fn strip_inline_code(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut chars = line.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '`' {
            // Find matching closing backtick
            let mut found_close = false;
            for (j, c2) in &mut chars {
                if c2 == '`' {
                    // Replace the code span with spaces to preserve positions
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

pub struct BoldItalicsRule;

impl Rule for BoldItalicsRule {
    fn name(&self) -> &str {
        "bold_italics"
    }

    fn description(&self) -> &str {
        "Checks that SKILL.md files do not use bold or italic markdown formatting"
    }

    fn run(&self, input: &str) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_idx, line) in input.lines().enumerate() {
            let line_num = (line_idx + 1) as u32;
            let trimmed = line.trim();

            // Toggle fenced code block state
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Skip bullet list items: lines starting with "* "
            if trimmed.starts_with("* ") {
                continue;
            }

            // Skip horizontal rules: lines that are only asterisks and optional spaces
            if !trimmed.is_empty()
                && trimmed.chars().all(|c| c == '*' || c == ' ')
                && trimmed.chars().filter(|&c| c == '*').count() >= 3
            {
                continue;
            }

            // Strip inline code spans before checking patterns
            let check_line = strip_inline_code(line);

            // Check for **bold** (must check before *italic* since ** contains *)
            self.find_double_star_patterns(&check_line, line_num, &mut violations);

            // Check for __bold__
            self.find_double_underscore_patterns(&check_line, line_num, &mut violations);

            // Check for *italic* (single star, but not inside already-matched **)
            self.find_single_star_patterns(&check_line, line_num, &mut violations);
        }

        violations
    }
}

impl BoldItalicsRule {
    fn find_double_star_patterns(
        &self,
        line: &str,
        line_num: u32,
        violations: &mut Vec<RuleViolation>,
    ) {
        let mut search_from = 0;
        while let Some(open) = line[search_from..].find("**") {
            let abs_open = search_from + open;
            // Skip if this is actually *** (triple star) — we'll still flag it,
            // just detect it as a ** containing content that starts with *
            let after_open = abs_open + 2;
            if after_open >= line.len() {
                break;
            }
            if let Some(close) = line[after_open..].find("**") {
                let inner = &line[after_open..after_open + close];
                if !inner.is_empty() && !inner.chars().all(|c| c == '*' || c == ' ') {
                    let matched = &line[abs_open..after_open + close + 2];
                    violations.push(RuleViolation {
                        message: format!("Bold/italic formatting found: {}", matched),
                        line: Some(line_num),
                        snippet: Some(line.to_string()),
                    });
                    search_from = after_open + close + 2;
                } else {
                    search_from = after_open;
                }
            } else {
                break;
            }
        }
    }

    fn find_double_underscore_patterns(
        &self,
        line: &str,
        line_num: u32,
        violations: &mut Vec<RuleViolation>,
    ) {
        let mut search_from = 0;
        while let Some(open) = line[search_from..].find("__") {
            let abs_open = search_from + open;
            let after_open = abs_open + 2;
            if after_open >= line.len() {
                break;
            }
            if let Some(close) = line[after_open..].find("__") {
                let inner = &line[after_open..after_open + close];
                if !inner.is_empty() {
                    let matched = &line[abs_open..after_open + close + 2];
                    violations.push(RuleViolation {
                        message: format!("Bold/italic formatting found: {}", matched),
                        line: Some(line_num),
                        snippet: Some(line.to_string()),
                    });
                    search_from = after_open + close + 2;
                } else {
                    search_from = after_open;
                }
            } else {
                break;
            }
        }
    }

    fn find_single_star_patterns(
        &self,
        line: &str,
        line_num: u32,
        violations: &mut Vec<RuleViolation>,
    ) {
        let mut search_from = 0;
        while search_from < line.len() {
            if let Some(open) = line[search_from..].find('*') {
                let abs_open = search_from + open;
                // Skip if this is part of a ** (already handled)
                if abs_open + 1 < line.len() && line.as_bytes()[abs_open + 1] == b'*' {
                    search_from = abs_open + 2;
                    continue;
                }
                // Skip if preceded by * (tail end of **)
                if abs_open > 0 && line.as_bytes()[abs_open - 1] == b'*' {
                    search_from = abs_open + 1;
                    continue;
                }
                let after_open = abs_open + 1;
                if after_open >= line.len() {
                    break;
                }
                // Find closing single * (not **)
                let mut found_close = false;
                let mut close_search = after_open;
                while close_search < line.len() {
                    if let Some(close) = line[close_search..].find('*') {
                        let abs_close = close_search + close;
                        // Check it's not part of **
                        let is_double_after =
                            abs_close + 1 < line.len() && line.as_bytes()[abs_close + 1] == b'*';
                        let is_double_before =
                            abs_close > 0 && line.as_bytes()[abs_close - 1] == b'*';
                        if is_double_after || is_double_before {
                            close_search = abs_close + 1;
                            continue;
                        }
                        let inner = &line[after_open..abs_close];
                        if !inner.is_empty() {
                            let matched = &line[abs_open..abs_close + 1];
                            violations.push(RuleViolation {
                                message: format!("Bold/italic formatting found: {}", matched),
                                line: Some(line_num),
                                snippet: Some(line.to_string()),
                            });
                            search_from = abs_close + 1;
                            found_close = true;
                            break;
                        } else {
                            close_search = abs_close + 1;
                        }
                    } else {
                        break;
                    }
                }
                if !found_close {
                    search_from = after_open;
                }
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_empty_for_clean_content() {
        let rule = BoldItalicsRule;
        let content = "---\nname: Test\n---\n\n<required>\nPlain text content.\n</required>\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn flags_double_star_bold() {
        let rule = BoldItalicsRule;
        let content = "Some text with **bold** in it.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(1));
        assert!(result[0].snippet.as_ref().unwrap().contains("**bold**"));
    }

    #[test]
    fn flags_single_star_italic() {
        let rule = BoldItalicsRule;
        let content = "Some text with *italic* in it.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(1));
        assert!(result[0].snippet.as_ref().unwrap().contains("*italic*"));
    }

    #[test]
    fn flags_triple_star_bold_italic() {
        let rule = BoldItalicsRule;
        let content = "Some text with ***bold italic*** in it.\n";
        let result = rule.run(content);
        assert!(!result.is_empty());
    }

    #[test]
    fn flags_double_underscore_bold() {
        let rule = BoldItalicsRule;
        let content = "Some text with __bold__ in it.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(1));
        assert!(result[0].snippet.as_ref().unwrap().contains("__bold__"));
    }

    #[test]
    fn does_not_flag_bullet_list_items() {
        let rule = BoldItalicsRule;
        let content = "* item one\n* item two\n* item three\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_horizontal_rule() {
        let rule = BoldItalicsRule;
        let content = "Some text.\n\n***\n\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_horizontal_rule_with_spaces() {
        let rule = BoldItalicsRule;
        let content = "Some text.\n\n* * *\n\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn reports_multiple_violations_across_lines() {
        let rule = BoldItalicsRule;
        let content = "First line with **bold** here.\nSecond line with *italic* here.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].line, Some(1));
        assert_eq!(result[1].line, Some(2));
    }

    #[test]
    fn reports_multiple_violations_on_same_line() {
        let rule = BoldItalicsRule;
        let content = "Text with **bold** and *italic* on same line.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].line, Some(1));
        assert_eq!(result[1].line, Some(1));
    }

    #[test]
    fn does_not_flag_single_asterisk_alone() {
        let rule = BoldItalicsRule;
        let content = "Use * for multiplication.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_double_asterisk_alone() {
        let rule = BoldItalicsRule;
        let content = "Use ** for exponentiation.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_bold_inside_fenced_code_block() {
        let rule = BoldItalicsRule;
        let content = "Normal text.\n```\necho \"**not bold**\"\n```\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn does_not_flag_bold_inside_fenced_code_block_with_language() {
        let rule = BoldItalicsRule;
        let content = "Normal text.\n```bash\necho \"**not bold**\"\n```\nMore text.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn flags_bold_outside_code_block_but_not_inside() {
        let rule = BoldItalicsRule;
        let content = "This has **bold** text.\n```\necho \"**not bold**\"\n```\nMore text.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(1));
    }

    #[test]
    fn does_not_flag_bold_inside_inline_code() {
        let rule = BoldItalicsRule;
        let content = "Use `**bold**` for bold formatting.\n";
        assert!(rule.run(content).is_empty());
    }

    #[test]
    fn flags_bold_outside_inline_code() {
        let rule = BoldItalicsRule;
        let content = "Use `code` and **bold** together.\n";
        let result = rule.run(content);
        assert_eq!(result.len(), 1);
        assert!(result[0].message.contains("**bold**"));
    }

    #[test]
    fn has_correct_name() {
        let rule = BoldItalicsRule;
        assert_eq!(rule.name(), "bold_italics");
    }

    #[test]
    fn has_non_empty_description() {
        let rule = BoldItalicsRule;
        assert!(!rule.description().is_empty());
    }
}
