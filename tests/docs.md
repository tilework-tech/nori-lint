# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` binary
- Tests invoke the compiled binary as a subprocess in temporary directories with controlled SKILL.md fixtures

### How it fits into the larger codebase

- Exercises the compiled binary produced from `@/src/main.rs` -- tests the full end-to-end pipeline including file discovery, rule execution, output formatting (text and JSON), and exit codes
- Run via `cargo test` -- Cargo automatically discovers files in the `tests/` directory as integration test targets
- Unit tests for the `Rule` trait and individual rules live in `@/src/registry.rs` and `@/src/rules/` alongside the production code

### Core Implementation

- `cli.rs` uses `tempfile::TempDir` to create isolated filesystem environments, writes SKILL.md files with known content, then runs the `nori-lint` binary via `assert_cmd` with `current_dir` set to the temp directory
- Tests are organized into groups: default/text format tests (backward compatibility, file discovery, mixed valid/invalid files, nested directories) and `--format json` tests (valid JSON output shape, empty array for no violations, multiple diagnostics, JSON field assertions)
- Helper functions `small_skill_content()` and `large_skill_content()` generate test fixtures -- the large fixture produces 200 lines, exceeding the 150-line limit
- JSON tests parse stdout with `serde_json` and assert on the array structure and individual diagnostic fields (`rule`, `file`, `message`, `line`, `snippet`)

### Things to Know

- `cargo_bin_cmd!("nori-lint")` resolves the binary path using the crate name from `@/Cargo.toml` -- if the package name changes, the macro argument must be updated
- Tests use `predicate::str::contains` for text output assertions, checking for rule names and file paths rather than exact output strings
- The `--format text` tests verify that explicit `--format text` produces identical output to the default (no flag) behavior
- The `--format json` tests verify the JSON contract: output is always an array of diagnostic objects, even when empty; each object has `rule`, `file`, `message`, `line`, and `snippet` keys
- Invalid `--format` values (e.g., `xml`) produce an error on stderr and exit code 1
- The mixed-files test accounts for OS path separator differences by checking for both `bad/SKILL.md` and `bad\SKILL.md`

Created and maintained by Nori.
