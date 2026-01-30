use crate::rules::Rule;

pub struct Registry {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassingRule;
    impl Rule for PassingRule {
        fn name(&self) -> &str {
            "passing"
        }
        fn description(&self) -> &str {
            "always passes"
        }
        fn run(&self, _input: &str) -> Option<String> {
            None
        }
    }

    struct FailingRule;
    impl Rule for FailingRule {
        fn name(&self) -> &str {
            "failing"
        }
        fn description(&self) -> &str {
            "always fails"
        }
        fn run(&self, _input: &str) -> Option<String> {
            Some("something went wrong".to_string())
        }
    }

    #[test]
    fn new_registry_has_no_rules() {
        let registry = Registry::new();
        assert!(registry.rules().is_empty());
    }

    #[test]
    fn register_adds_rule_to_registry() {
        let mut registry = Registry::new();
        registry.register(Box::new(PassingRule));
        assert_eq!(registry.rules().len(), 1);
        assert_eq!(registry.rules()[0].name(), "passing");
    }

    #[test]
    fn registered_rules_run_correctly() {
        let mut registry = Registry::new();
        registry.register(Box::new(PassingRule));
        registry.register(Box::new(FailingRule));

        let rules = registry.rules();
        assert_eq!(rules.len(), 2);
        assert!(rules[0].run("anything").is_none());
        assert!(rules[1].run("anything").is_some());
    }
}
