pub mod line_count;
pub mod llm_rules;
pub mod required_tags;
pub mod unclosed_tags;

use crate::diagnostic::RuleViolation;

pub trait Rule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, input: &str) -> Option<RuleViolation>;
}
