# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` binary
- Tests invoke the compiled binary as a subprocess in temporary directories with controlled SKILL.md fixtures

### How it fits into the larger codebase

- Exercises the compiled binary produced from `@/src/main.rs` -- tests the full end-to-end pipeline including file discovery, rule execution, output formatting, and exit codes
- Run via `cargo test` -- Cargo automatically discovers files in the `tests/` directory as integration test targets
- Unit tests for the `Rule` trait and individual rules live in `@/src/registry.rs` and `@/src/rules/` alongside the production code

### Core Implementation

- `cli.rs` uses `tempfile::TempDir` to create isolated filesystem environments, writes SKILL.md files with known content, then runs the `nori-lint` binary via `assert_cmd` with `current_dir` set to the temp directory
- Tests cover: no SKILL.md files present (exit 0), valid files (exit 0), files exceeding the line limit (exit 1), mixed valid/invalid files (only violating files appear in output), and nested directory discovery
- Helper functions `small_skill_content()` and `large_skill_content()` generate test fixtures -- the large fixture produces 200 lines, exceeding the 150-line limit

### Things to Know

- `cargo_bin_cmd!("nori-lint")` resolves the binary path using the crate name from `@/Cargo.toml` -- if the package name changes, the macro argument must be updated
- Tests use `predicate::str::contains` for output assertions, checking for rule names and file paths rather than exact output strings
- The mixed-files test accounts for OS path separator differences by checking for both `bad/SKILL.md` and `bad\SKILL.md`

Created and maintained by Nori.
