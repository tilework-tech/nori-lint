# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` CLI
- Tests invoke the `run()` function directly with stubbed `process.argv`, `process.stdout.write`, and `process.stderr.write` to capture output

### How it fits into the larger codebase

- Exercises the full end-to-end pipeline for all three subcommands (`lint`, `fix`, and `list`): file discovery, rule execution, rule enable/disable filtering, output formatting (text and JSON), directory argument handling, `--help` output, config handling, dry-run mode, rule metadata listing, and exit codes
- Run via `npm test` (vitest)
- Unit tests for individual rules and modules live alongside production code in `@/src/` as `.test.ts` files
- LLM rules are not exercised via real API calls in integration tests; config tests verify config loading, validation, and error reporting behavior

### Core Implementation

- `cli.test.ts` uses `fs.mkdtempSync()` to create isolated filesystem environments, writes SKILL.md files with known content, then calls `run()` with stubbed `process.argv`
- A `withArgs()` helper stubs `process.argv`, captures stdout/stderr, and returns `{ code, stdout, stderr }`
- Helper functions create SKILL.md fixtures targeting specific rule behaviors
- Tests are organized into groups covering: default behavior (no subcommand shows help), lint subcommand (basic behavior, directory argument handling, `--format text`/`--format json` output, `--help`, per-rule violation detection, `--config` behavior, rules config enable/disable, `.nori-lint.json` auto-discovery, unknown rule warnings), fix subcommand (applying deterministic fixes, unfixable rule reporting, code block content preservation during fix, `--dry-run` mode, `--config` interactions), list subcommand (text format output with all rule names/descriptions/tags, JSON format output with structured rule metadata, invalid format error handling, `--help`), and entry point symlink invocation (subprocess tests that verify the CLI works when invoked via a symlink, as npm does)
- The entry point tests use `execFileSync` to spawn the built CLI as a real subprocess, unlike all other tests which call `run()` in-process. The `entry point - npm pack simulation` group goes further by running `npm pack`, extracting the tarball, installing production dependencies, creating a symlink, and running the CLI -- this simulates exactly what `npm install -g nori-lint` does and catches stale build artifacts in the published tarball. The `entry point - symlink invocation` group tests symlink and direct invocation of the build output without packing.

### Things to Know

- The config integration tests validate error paths only -- they do not test successful LLM rule execution since that would require a real API key
- The auto-discovery test writes a deliberately invalid config to prove the file was loaded
- Tests that check for a specific rule's violation use `.find()` or `.some()` on the diagnostics array rather than asserting exact array length, since a single file may trigger violations from multiple rules simultaneously
- Temp directories are cleaned up in `afterEach` to prevent disk accumulation
- The entry point tests (both symlink invocation and npm-pack simulation) are the only tests that exercise the actual entry point guard in `cli.ts`; they require a prior build (`npm run build`) since they run the compiled output as a subprocess, and use `test.skipIf(!hasBuild)` to skip gracefully when no build exists

Created and maintained by Nori.
