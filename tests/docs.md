# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` binary
- Tests invoke the compiled binary as a subprocess in temporary directories with controlled SKILL.md fixtures

### How it fits into the larger codebase

- Exercises the compiled binary produced from `@/src/main.rs` -- tests the full end-to-end pipeline including file discovery, rule execution, output formatting (text and JSON), directory argument handling, `--help` output, and exit codes
- Run via `cargo test` -- Cargo automatically discovers files in the `tests/` directory as integration test targets
- Unit tests for the `Rule` trait and individual rules live in `@/src/rules/` alongside the production code in `#[cfg(test)]` modules

### Core Implementation

- `cli.rs` uses `tempfile::TempDir` to create isolated filesystem environments, writes SKILL.md files with known content, then runs the `nori-lint` binary via `assert_cmd`
- Tests exercise the binary in two modes: via `current_dir` (setting the working directory for implicit root) and via `.arg()` (passing an explicit directory path argument)
- Helper functions create SKILL.md fixtures that target specific rule behaviors: valid skills (small, well-formed), oversized skills (exceeding 150-line limit), skills missing `<required>` tags, and skills with unclosed tags
- Tests are organized into groups covering: default/text format output, directory argument handling, `--format json` output shape and field validation, `--help`/`-h` flag behavior, and per-rule violation detection for each registered rule
- JSON tests parse stdout with `serde_json` and assert on the array structure and individual diagnostic fields (`rule`, `file`, `message`, `line`, `snippet`)
- Tests that check for a specific rule's violation use `.find()` on the diagnostics array rather than asserting exact array length, since a single file may trigger violations from multiple rules simultaneously

### Things to Know

- `cargo_bin_cmd!("nori-lint")` resolves the binary path using the crate name from `@/Cargo.toml` -- if the package name changes, the macro argument must be updated
- Tests use `predicate::str::contains` for text output assertions, checking for rule names and file paths rather than exact output strings
- The `small_skill_content()` helper includes `<required>...</required>` tags so it passes all rules, not just the line count rule
- A file can trigger violations from multiple rules at once (e.g., a file missing `<required>` tags also triggers `required_tags` while potentially triggering `unclosed_tags` if it has orphan tags) -- tests account for this by filtering diagnostics by rule name
- The mixed-files test accounts for OS path separator differences by checking for both `bad/SKILL.md` and `bad\SKILL.md`
- `--help` tests verify that help output includes usage syntax, the `--format` option, and all registered rule names; they also verify `-h` produces identical output to `--help`, and that `--help` takes priority over other arguments (exits 0 without linting)
- CLI parse errors (e.g., `--format` with a missing or invalid value) produce clap-generated error messages on stderr and exit code 2

Created and maintained by Nori.
