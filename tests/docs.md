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

- `cli.rs` uses `tempfile::TempDir` to create isolated filesystem environments, writes SKILL.md files with known content, then runs the `nori-lint` binary via `assert_cmd`
- Tests exercise the binary in two modes: via `current_dir` (setting the working directory for implicit root) and via `.arg()` (passing an explicit directory path argument)
- Tests cover: no SKILL.md files present (exit 0), valid files (exit 0), files exceeding the line limit (exit 1), mixed valid/invalid files (only violating files appear in output), nested directory discovery, explicit directory path arguments (both valid and with violations), and error handling for nonexistent directory paths
- Helper functions `small_skill_content()` and `large_skill_content()` generate test fixtures -- the large fixture produces 200 lines, exceeding the 150-line limit

### Things to Know

- `cargo_bin_cmd!("nori-lint")` resolves the binary path using the crate name from `@/Cargo.toml` -- if the package name changes, the macro argument must be updated
- Tests use `predicate::str::contains` for output assertions, checking for rule names and file paths rather than exact output strings
- The mixed-files test accounts for OS path separator differences by checking for both `bad/SKILL.md` and `bad\SKILL.md`
- The nonexistent directory test asserts on stderr (not stdout) since invalid directory errors are written to stderr

Created and maintained by Nori.
