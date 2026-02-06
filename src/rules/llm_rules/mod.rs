pub mod process_not_integration;
pub mod redundant_explanation;

use crate::diagnostic::RuleViolation;

pub trait LlmRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn evaluate(&self, input: &str, llm_response: &str) -> Option<RuleViolation>;
}
