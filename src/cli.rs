use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::{self, Config};
use crate::diagnostic::LintDiagnostic;
use crate::llm_client::{AnthropicClient, LlmAnalyzer};
use crate::llm_registry::LlmRegistry;
use crate::registry::Registry;
use crate::rules::line_count::LineCountRule;
use crate::rules::llm_rules::redundant_explanation::RedundantExplanationRule;
use crate::rules::required_tags::RequiredTagsRule;
use crate::rules::unclosed_tags::UnclosedTagsRule;

#[derive(Debug, PartialEq)]
enum OutputFormat {
    Text,
    Json,
}

fn match_format_value(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(format!("invalid format '{other}': expected text or json")),
    }
}

struct CliArgs {
    format: OutputFormat,
    root: String,
    config_path: Option<String>,
}

fn parse_args(args: &[String]) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut root = ".".to_string();
    let mut config_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if let Some(val) = args[i].strip_prefix("--format=") {
            format = match_format_value(val)?;
        } else if args[i] == "--format" {
            if i + 1 >= args.len() {
                return Err("--format requires a value (text or json)".to_string());
            }
            format = match_format_value(&args[i + 1])?;
            i += 1;
        } else if let Some(val) = args[i].strip_prefix("--config=") {
            config_path = Some(val.to_string());
        } else if args[i] == "--config" {
            if i + 1 >= args.len() {
                return Err("--config requires a value".to_string());
            }
            config_path = Some(args[i + 1].clone());
            i += 1;
        } else if !args[i].starts_with('-') {
            root = args[i].clone();
        }
        i += 1;
    }
    Ok(CliArgs {
        format,
        root,
        config_path,
    })
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
    content: &str,
    display_str: &str,
    diagnostics: &mut Vec<LintDiagnostic>,
    has_llm_error: &mut bool,
) {
    for rule in llm_registry.rules() {
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
    let args: Vec<String> = env::args().skip(1).collect();

    let cli = match parse_args(&args) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 1;
        }
    };

    let config = match resolve_config(cli.config_path.as_deref()) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("error: {msg}");
            return 1;
        }
    };

    let root_path = Path::new(&cli.root);

    if !root_path.exists() {
        eprintln!("error: {} does not exist", cli.root);
        return 1;
    } else if !root_path.is_dir() {
        eprintln!("error: {} is not a directory", cli.root);
        return 1;
    }

    let mut registry = Registry::new();
    registry.register(Box::new(LineCountRule));
    registry.register(Box::new(RequiredTagsRule));
    registry.register(Box::new(UnclosedTagsRule));

    let llm_client = config
        .as_ref()
        .map(|c| AnthropicClient::new(c.anthropic_api_key.clone()));
    let llm_registry = {
        let mut r = LlmRegistry::new();
        if config.is_some() {
            r.register(Box::new(RedundantExplanationRule));
        }
        r
    };

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
                        if let Some(violation) = rule.run(&content) {
                            diagnostics.push(LintDiagnostic::from_violation(
                                &violation,
                                rule.name(),
                                &display_str,
                            ));
                        }
                    }

                    if let Some(client) = &llm_client {
                        run_llm_rules(
                            client,
                            &llm_registry,
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
