use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::registry::Registry;
use crate::rules::line_count::LineCountRule;

pub fn run() -> i32 {
    let args: Vec<String> = env::args().collect();
    let root = if args.len() > 1 { &args[1] } else { "." };
    let root_path = Path::new(root);

    if !root_path.exists() {
        eprintln!("error: {root} does not exist");
        return 1;
    } else if !root_path.is_dir() {
        eprintln!("error: {root} is not a directory");
        return 1;
    }

    let mut registry = Registry::new();
    registry.register(Box::new(LineCountRule));

    let mut has_violations = false;

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "SKILL.md" {
            let path = entry.path();
            let display_path = path.strip_prefix(root_path).unwrap_or(path);
            match fs::read_to_string(path) {
                Ok(content) => {
                    for rule in registry.rules() {
                        if let Some(message) = rule.run(&content) {
                            println!("[{}] {}: {}", rule.name(), display_path.display(), message);
                            has_violations = true;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error: could not read {}: {e}", display_path.display());
                    has_violations = true;
                }
            }
        }
    }

    if has_violations { 1 } else { 0 }
}
