use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use clap::{Parser, ValueEnum};

use crate::config::{self, Config};
use crate::diagnostic::LintDiagnostic;
use crate::llm_client::{AnthropicClient, LlmAnalyzer};
use crate::llm_registry::LlmRegistry;
use crate::registry::Registry;
use crate::rules::bold_italics::BoldItalicsRule;
use crate::rules::line_count::LineCountRule;
use crate::rules::llm_rules::cli_command_index::CliCommandIndexRule;
use crate::rules::llm_rules::duplicate_section::DuplicateSectionRule;
use crate::rules::llm_rules::first_person::FirstPersonRule;
use crate::rules::llm_rules::negative_without_positive::NegativeWithoutPositiveRule;
use crate::rules::llm_rules::obvious_instructions::ObviousInstructionsRule;
use crate::rules::llm_rules::process_not_integration::ProcessNotIntegrationRule;
use crate::rules::llm_rules::redundant_explanation::RedundantExplanationRule;
use crate::rules::llm_rules::unexplained_url::UnexplainedUrlRule;
use crate::rules::markdown_links::MarkdownLinksRule;
use crate::rules::redundant_title::RedundantTitleRule;
use crate::rules::required_tags::RequiredTagsRule;
use crate::rules::unclosed_tags::UnclosedTagsRule;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn default_registry() -> Registry {
    let mut registry = Registry::new();
    registry.register(Box::new(BoldItalicsRule));
    registry.register(Box::new(LineCountRule));
    registry.register(Box::new(MarkdownLinksRule));
    registry.register(Box::new(RedundantTitleRule));
    registry.register(Box::new(RequiredTagsRule));
    registry.register(Box::new(UnclosedTagsRule));
    registry
}

/// Lint SKILL.md files for common issues
#[derive(Parser, Debug)]
#[command(name = "nori-lint")]
struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Path to config file (default: .nori-lint.json in current directory)
    #[arg(long)]
    config: Option<String>,

    /// Directory to lint
    #[arg(default_value = ".")]
    path: String,
}

fn resolve_config(cli_config_path: Option<&str>) -> Result<Option<Config>, String> {
    if let Some(path) = cli_config_path {
        let config = config::load_config(Path::new(path))?;
        return Ok(Some(config));
    }

    let cwd_config = Path::new(".nori-lint.json");
    if cwd_config.exists() {
        let config = config::load_config(cwd_config)?;
        return Ok(Some(config));
    }

    Ok(None)
}

pub(crate) async fn run_llm_rules<A: LlmAnalyzer>(
    client: &A,
    llm_registry: &LlmRegistry,
    config: &Config,
    content: &str,
    display_str: &str,
    diagnostics: &mut Vec<LintDiagnostic>,
    has_llm_error: &mut bool,
) {
    let enabled_rules: Vec<_> = llm_registry
        .rules()
        .iter()
        .filter(|rule| config.is_rule_enabled(rule.name()))
        .collect();

    if enabled_rules.is_empty() {
        return;
    }

    let rule_names: Vec<&str> = enabled_rules.iter().map(|r| r.name()).collect();
    eprintln!("  checking {}...", rule_names.join(", "));

    let futures = enabled_rules.iter().map(|rule| async move {
        let result = client.analyze(rule.system_prompt(), content).await;
        (rule, result)
    });

    let results = futures::future::join_all(futures).await;

    for (rule, result) in results {
        match result {
            Ok(response) => {
                if let Some(violation) = rule.evaluate(content, &response) {
                    diagnostics.push(LintDiagnostic::from_violation(
                        &violation,
                        rule.name(),
                        display_str,
                    ));
                }
            }
            Err(e) => {
                eprintln!(
                    "error: LLM rule '{}' failed for {}: {e}",
                    rule.name(),
                    display_str
                );
                *has_llm_error = true;
            }
        }
    }
}

pub async fn run() -> i32 {
    let cli = Cli::parse();

    let config = match resolve_config(cli.config.as_deref()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 1;
        }
    };

    let root_path = Path::new(&cli.path);

    if !root_path.exists() {
        eprintln!("error: {} does not exist", cli.path);
        return 1;
    } else if !root_path.is_dir() {
        eprintln!("error: {} is not a directory", cli.path);
        return 1;
    }

    let registry = default_registry();

    if config.is_none() {
        eprintln!("note: skipping LLM rules (no .nori-lint.json found; use --config to specify)");
    }

    let llm_client = config
        .as_ref()
        .map(|c| AnthropicClient::new(c.anthropic_api_key.clone()));
    let llm_registry = {
        let mut r = LlmRegistry::new();
        if config.is_some() {
            r.register(Box::new(CliCommandIndexRule));
            r.register(Box::new(DuplicateSectionRule));
            r.register(Box::new(FirstPersonRule));
            r.register(Box::new(NegativeWithoutPositiveRule));
            r.register(Box::new(ObviousInstructionsRule));
            r.register(Box::new(ProcessNotIntegrationRule));
            r.register(Box::new(RedundantExplanationRule));
            r.register(Box::new(UnexplainedUrlRule));
        }
        r
    };

    if let Some(cfg) = &config {
        let mut all_known_rules: Vec<&str> = registry.rules().iter().map(|r| r.name()).collect();
        all_known_rules.extend(llm_registry.rules().iter().map(|r| r.name()));

        if let Some(rules_config) = &cfg.rules {
            let names_to_check = rules_config
                .enabled
                .as_deref()
                .or(rules_config.disabled.as_deref())
                .unwrap_or(&[]);
            for name in names_to_check {
                if !all_known_rules.contains(&name.as_str()) {
                    eprintln!("warning: unknown rule '{}' in config", name);
                }
            }
        }
    }

    let mut diagnostics: Vec<LintDiagnostic> = Vec::new();
    let mut has_read_error = false;
    let mut has_llm_error = false;

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "SKILL.md" {
            let path = entry.path();
            let display_path = path.strip_prefix(root_path).unwrap_or(path);
            let display_str = display_path.display().to_string();
            match fs::read_to_string(path) {
                Ok(content) => {
                    for rule in registry.rules() {
                        if config
                            .as_ref()
                            .is_some_and(|c| !c.is_rule_enabled(rule.name()))
                        {
                            continue;
                        }
                        for violation in rule.run(&content) {
                            diagnostics.push(LintDiagnostic::from_violation(
                                &violation,
                                rule.name(),
                                &display_str,
                            ));
                        }
                    }

                    if let (Some(client), Some(cfg)) = (&llm_client, config.as_ref()) {
                        run_llm_rules(
                            client,
                            &llm_registry,
                            cfg,
                            &content,
                            &display_str,
                            &mut diagnostics,
                            &mut has_llm_error,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    eprintln!("error: could not read {}: {e}", display_str);
                    has_read_error = true;
                }
            }
        }
    }

    let has_violations = !diagnostics.is_empty() || has_read_error || has_llm_error;

    match cli.format {
        OutputFormat::Text => {
            for diag in &diagnostics {
                match diag.line {
                    Some(line) => {
                        println!("[{}] {}:{}: {}", diag.rule, diag.file, line, diag.message)
                    }
                    None => println!("[{}] {}: {}", diag.rule, diag.file, diag.message),
                }
                if let Some(snippet) = &diag.snippet {
                    println!("  | {snippet}");
                }
            }
        }
        OutputFormat::Json => match serde_json::to_string(&diagnostics) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("error: failed to serialize diagnostics: {e}");
                return 1;
            }
        },
    }

    if has_violations { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, RulesConfig};
    use crate::diagnostic::RuleViolation;
    use crate::llm_client::{LlmAnalyzer, LlmError};
    use crate::llm_registry::LlmRegistry;
    use crate::rules::llm_rules::LlmRule;
    use std::time::Duration;

    struct DelayMockAnalyzer {
        delay: Duration,
        response: Result<String, String>,
    }

    impl LlmAnalyzer for DelayMockAnalyzer {
        async fn analyze(
            &self,
            _system_prompt: &str,
            _user_content: &str,
        ) -> Result<String, LlmError> {
            tokio::time::sleep(self.delay).await;
            self.response
                .clone()
                .map_err(|e| LlmError::HttpError(e.clone()))
        }
    }

    struct AlwaysViolatesRule {
        rule_name: &'static str,
    }

    impl LlmRule for AlwaysViolatesRule {
        fn name(&self) -> &str {
            self.rule_name
        }
        fn description(&self) -> &str {
            "mock rule"
        }
        fn system_prompt(&self) -> &str {
            "mock prompt"
        }
        fn evaluate(&self, _input: &str, _llm_response: &str) -> Option<RuleViolation> {
            Some(RuleViolation {
                message: format!("violation from {}", self.rule_name),
                line: None,
                snippet: None,
            })
        }
    }

    fn make_config(rules: Option<RulesConfig>) -> Config {
        Config {
            anthropic_api_key: "fake-key".to_string(),
            rules,
        }
    }

    #[tokio::test]
    async fn collects_diagnostics_from_all_rules() {
        let client = DelayMockAnalyzer {
            delay: Duration::from_millis(10),
            response: Ok("{}".to_string()),
        };
        let mut registry = LlmRegistry::new();
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_a",
        }));
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_b",
        }));
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_c",
        }));

        let config = make_config(None);
        let mut diagnostics = Vec::new();
        let mut has_llm_error = false;

        run_llm_rules(
            &client,
            &registry,
            &config,
            "file content",
            "test.md",
            &mut diagnostics,
            &mut has_llm_error,
        )
        .await;

        assert_eq!(
            diagnostics.len(),
            3,
            "should collect diagnostics from all 3 rules"
        );
        assert!(!has_llm_error);
        let rules: Vec<&str> = diagnostics.iter().map(|d| d.rule.as_str()).collect();
        assert!(rules.contains(&"rule_a"));
        assert!(rules.contains(&"rule_b"));
        assert!(rules.contains(&"rule_c"));
    }

    #[tokio::test]
    async fn error_in_one_rule_does_not_block_others() {
        // The analyzer always errors, but we have 3 rules.
        // Two rules always-violate, one never-violates.
        // With an error response, none should produce diagnostics,
        // but all should be attempted (has_llm_error = true).
        let client = DelayMockAnalyzer {
            delay: Duration::from_millis(10),
            response: Err("api failure".to_string()),
        };
        let mut registry = LlmRegistry::new();
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_a",
        }));
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_b",
        }));
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_c",
        }));

        let config = make_config(None);
        let mut diagnostics = Vec::new();
        let mut has_llm_error = false;

        run_llm_rules(
            &client,
            &registry,
            &config,
            "file content",
            "test.md",
            &mut diagnostics,
            &mut has_llm_error,
        )
        .await;

        assert!(has_llm_error, "should flag llm error");
        assert!(
            diagnostics.is_empty(),
            "errors should produce no diagnostics"
        );
    }

    #[tokio::test]
    async fn disabled_rules_are_skipped() {
        let client = DelayMockAnalyzer {
            delay: Duration::from_millis(10),
            response: Ok("{}".to_string()),
        };
        let mut registry = LlmRegistry::new();
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_a",
        }));
        registry.register(Box::new(AlwaysViolatesRule {
            rule_name: "rule_b",
        }));

        let config = make_config(Some(RulesConfig {
            enabled: None,
            disabled: Some(vec!["rule_a".to_string()]),
        }));
        let mut diagnostics = Vec::new();
        let mut has_llm_error = false;

        run_llm_rules(
            &client,
            &registry,
            &config,
            "file content",
            "test.md",
            &mut diagnostics,
            &mut has_llm_error,
        )
        .await;

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "rule_b");
    }
}
