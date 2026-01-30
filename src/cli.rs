use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::registry::Registry;
use crate::rules::line_count::LineCountRule;

pub fn run() -> i32 {
    let mut registry = Registry::new();
    registry.register(Box::new(LineCountRule));

    let mut has_violations = false;

    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "SKILL.md" {
            let path = entry.path();
            let display_path = path.strip_prefix(Path::new(".")).unwrap_or(path);
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
