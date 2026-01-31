use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use clap::{Parser, ValueEnum};

use crate::diagnostic::LintDiagnostic;
use crate::registry::Registry;
use crate::rules::line_count::LineCountRule;
use crate::rules::required_tags::RequiredTagsRule;
use crate::rules::unclosed_tags::UnclosedTagsRule;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn default_registry() -> Registry {
    let mut registry = Registry::new();
    registry.register(Box::new(LineCountRule));
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

    /// Directory to lint
    #[arg(default_value = ".")]
    path: String,
}

pub fn run() -> i32 {
    let cli = Cli::parse();

    let root_path = Path::new(&cli.path);

    if !root_path.exists() {
        eprintln!("error: {} does not exist", cli.path);
        return 1;
    } else if !root_path.is_dir() {
        eprintln!("error: {} is not a directory", cli.path);
        return 1;
    }

    let registry = default_registry();

    let mut diagnostics: Vec<LintDiagnostic> = Vec::new();
    let mut has_read_error = false;

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
                }
                Err(e) => {
                    eprintln!("error: could not read {}: {e}", display_str);
                    has_read_error = true;
                }
            }
        }
    }

    let has_violations = !diagnostics.is_empty() || has_read_error;

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
