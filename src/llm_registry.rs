use crate::rules::llm_rules::LlmRule;

pub struct LlmRegistry {
    rules: Vec<Box<dyn LlmRule>>,
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register(&mut self, rule: Box<dyn LlmRule>) {
        self.rules.push(rule);
    }

    pub fn rules(&self) -> &[Box<dyn LlmRule>] {
        &self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::RuleViolation;

    struct MockLlmRule;
    impl LlmRule for MockLlmRule {
        fn name(&self) -> &str {
            "mock_llm"
        }
        fn description(&self) -> &str {
            "a mock LLM rule"
        }
        fn system_prompt(&self) -> &str {
            "you are a mock"
        }
        fn evaluate(&self, _input: &str, _llm_response: &str) -> Option<RuleViolation> {
            None
        }
    }

    #[test]
    fn new_registry_has_no_rules() {
        let registry = LlmRegistry::new();
        assert!(registry.rules().is_empty());
    }

    #[test]
    fn register_adds_rule() {
        let mut registry = LlmRegistry::new();
        registry.register(Box::new(MockLlmRule));
        assert_eq!(registry.rules().len(), 1);
        assert_eq!(registry.rules()[0].name(), "mock_llm");
    }
}
