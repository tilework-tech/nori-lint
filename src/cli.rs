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
use crate::rules::llm_rules::first_person::FirstPersonRule;
use crate::rules::llm_rules::negative_without_positive::NegativeWithoutPositiveRule;
use crate::rules::llm_rules::obvious_instructions::ObviousInstructionsRule;
use crate::rules::llm_rules::process_not_integration::ProcessNotIntegrationRule;
use crate::rules::llm_rules::redundant_explanation::RedundantExplanationRule;
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
    registry.register(Box::new(RedundantTitleRule));
    registry.register(Box::new(RequiredTagsRule));
    registry.register(Box::new(UnclosedTagsRule));
    registry
}

fn build_rules_help() -> String {
    let registry = default_registry();
    let mut lines = vec!["Rules:".to_string()];
    for rule in registry.rules() {
        lines.push(format!("  {:20} {}", rule.name(), rule.description()));
    }
    lines.join("\n")
}

/// Lint SKILL.md files for common issues
#[derive(Parser, Debug)]
#[command(name = "nori-lint", after_help = build_rules_help())]
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

async fn run_llm_rules<A: LlmAnalyzer>(
    client: &A,
    llm_registry: &LlmRegistry,
    config: &Config,
    content: &str,
    display_str: &str,
    diagnostics: &mut Vec<LintDiagnostic>,
    has_llm_error: &mut bool,
) {
    for rule in llm_registry.rules() {
        if !config.is_rule_enabled(rule.name()) {
            continue;
        }
        match client.analyze(rule.system_prompt(), content).await {
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
            r.register(Box::new(FirstPersonRule));
            r.register(Box::new(NegativeWithoutPositiveRule));
            r.register(Box::new(ObviousInstructionsRule));
            r.register(Box::new(ProcessNotIntegrationRule));
            r.register(Box::new(RedundantExplanationRule));
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
