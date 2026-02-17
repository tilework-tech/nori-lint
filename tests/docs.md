# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` CLI
- Tests invoke the `run()` function directly with stubbed `process.argv`, `process.stdout.write`, and `process.stderr.write` to capture output

### How it fits into the larger codebase

- Exercises the full end-to-end pipeline: file discovery, rule execution, rule enable/disable filtering, output formatting (text and JSON), directory argument handling, `--help` output, config handling, and exit codes
- Run via `npm test` (vitest)
- Unit tests for individual rules and modules live alongside production code in `@/src/` as `.test.ts` files
- LLM rules are not exercised via real API calls in integration tests; config tests verify config loading, validation, and error reporting behavior

### Core Implementation

- `cli.test.ts` uses `fs.mkdtempSync()` to create isolated filesystem environments, writes SKILL.md files with known content, then calls `run()` with stubbed `process.argv`
- A `withArgs()` helper stubs `process.argv`, captures stdout/stderr, and returns `{ code, stdout, stderr }`
- Helper functions create SKILL.md fixtures targeting specific rule behaviors
- Tests are organized into groups covering: default behavior (no subcommand shows help), lint subcommand basic behavior, directory argument handling, `--format text` output, `--format json` output, `--help`, per-rule violation detection, `--config` behavior, rules config enable/disable, `.nori-lint.json` auto-discovery, and unknown rule warnings

### Things to Know

- The config integration tests validate error paths only -- they do not test successful LLM rule execution since that would require a real API key
- The auto-discovery test writes a deliberately invalid config to prove the file was loaded
- Tests that check for a specific rule's violation use `.find()` or `.some()` on the diagnostics array rather than asserting exact array length, since a single file may trigger violations from multiple rules simultaneously
- Temp directories are cleaned up in `afterEach` to prevent disk accumulation

Created and maintained by Nori.
