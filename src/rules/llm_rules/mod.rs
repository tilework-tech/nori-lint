pub mod cli_command_index;
pub mod duplicate_section;
pub mod first_person;
pub mod negative_without_positive;
pub mod obvious_instructions;
pub mod process_not_integration;
pub mod redundant_explanation;
pub mod unexplained_url;

use serde::Deserialize;

use crate::diagnostic::RuleViolation;

#[derive(Debug, Clone, Deserialize)]
pub struct LlmResponse {
    pub has_violations: bool,
    pub violations: Vec<LlmViolation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmViolation {
    pub text: String,
    pub reason: String,
}

pub trait LlmRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn system_prompt(&self) -> &str;
    fn evaluate(&self, input: &str, violations: &[LlmViolation]) -> Option<RuleViolation>;
}
