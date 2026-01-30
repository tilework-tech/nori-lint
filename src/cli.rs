use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::diagnostic::LintDiagnostic;
use crate::registry::Registry;
use crate::rules::line_count::LineCountRule;

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
}

fn parse_args(args: &[String]) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut root = ".".to_string();
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
        } else if !args[i].starts_with('-') {
            root = args[i].clone();
        }
        i += 1;
    }
    Ok(CliArgs { format, root })
}

pub fn run() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();

    let cli = match parse_args(&args) {
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
