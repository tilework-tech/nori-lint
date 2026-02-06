# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` binary
- Tests invoke the compiled binary as a subprocess in temporary directories with controlled SKILL.md fixtures

### How it fits into the larger codebase

- Exercises the compiled binary produced from `@/src/main.rs` -- tests the full end-to-end pipeline including file discovery, rule execution, output formatting (text and JSON), directory argument handling, `--help` output, config handling, and exit codes
- Run via `cargo test` -- Cargo automatically discovers files in the `tests/` directory as integration test targets
- Unit tests for the `Rule` trait, `LlmRule` trait, and individual rules live in `@/src/` alongside the production code in `#[cfg(test)]` modules
- LLM rules are not exercised via real API calls in integration tests; the config/`--config` tests verify config loading, validation, and error reporting behavior

### Core Implementation

- `cli.rs` uses `tempfile::TempDir` to create isolated filesystem environments, writes SKILL.md files with known content, then runs the `nori-lint` binary via `assert_cmd`
- Tests exercise the binary in two modes: via `current_dir` (setting the working directory for implicit root) and via `.arg()` (passing an explicit directory path argument)
- Helper functions create SKILL.md fixtures that target specific rule behaviors: valid skills (small, well-formed), oversized skills (exceeding 150-line limit), skills missing `<required>` tags, skills with unclosed tags, skills with bold/italic markdown formatting, and skills with redundant title headings
- Tests are organized into groups covering: default/text format output, directory argument handling, `--format json` output shape and field validation, `--help`/`-h` flag behavior, per-rule violation detection, and `--config` flag behavior
- Config integration tests verify: deterministic rules still run without config (with stderr note), `--config` with nonexistent file errors, invalid JSON errors, missing `anthropic_api_key` field errors, `--config=` equals syntax, `--config` with missing value, and auto-discovery of `.nori-lint.json` in the working directory
- JSON tests parse stdout with `serde_json` and assert on the array structure and individual diagnostic fields (`rule`, `file`, `message`, `line`, `snippet`)
- Tests that check for a specific rule's violation use `.find()` on the diagnostics array rather than asserting exact array length, since a single file may trigger violations from multiple rules simultaneously
- The `bold_italics` integration tests verify that the rule appears in `--help`, that bold text in a SKILL.md causes a failure exit, and that JSON output includes `line` and `snippet` fields for bold_italics violations

### Things to Know

- The config integration tests validate error paths only -- they do not test successful LLM rule execution since that would require a real Anthropic API key and network access
- The auto-discovery test (`nori_lint_json_in_cwd_is_auto_discovered`) writes a deliberately invalid config to prove the file was loaded: the test expects a failure with an `anthropic_api_key` error message
- `cargo_bin_cmd!("nori-lint")` resolves the binary path using the crate name from `@/Cargo.toml` -- if the package name changes, the macro argument must be updated
- Tests use `predicate::str::contains` for text output assertions, checking for rule names and file paths rather than exact output strings
- The `small_skill_content()` helper includes `<required>...</required>` tags so it passes all deterministic rules
- A file can trigger violations from multiple rules at once -- tests account for this by filtering diagnostics by rule name
- The mixed-files test accounts for OS path separator differences by checking for both `bad/SKILL.md` and `bad\SKILL.md`
- `--help` tests verify that help output includes usage syntax, the `--format` option, and all registered rule names; they also verify `-h` produces identical output to `--help`, and that `--help` takes priority over other arguments (exits 0 without linting)
- CLI parse errors (e.g., `--format` with a missing or invalid value) produce clap-generated error messages on stderr and exit code 2

Created and maintained by Nori.
