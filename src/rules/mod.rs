pub mod line_count;

use crate::diagnostic::RuleViolation;

pub trait Rule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, input: &str) -> Option<RuleViolation>;
}
