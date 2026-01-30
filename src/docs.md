# Noridoc: src

Path: @/src

### Overview

- Contains the library and binary source for the `nori-lint` CLI
- Organized into a hybrid crate: `main.rs` (thin binary entry point) and `lib.rs` (library exposing `cli`, `registry`, and `rules` modules)

### How it fits into the larger codebase

- `main.rs` is the binary entry point compiled into the `nori-lint` executable; it delegates immediately to `cli::run()` in the library
- Integration tests in `@/tests/cli.rs` exercise the compiled binary as a subprocess
- Unit tests live alongside production code in `registry.rs` and `rules/line_count.rs` via `#[cfg(test)]` modules

### Core Implementation

- **`main.rs`** -- Calls `nori_lint::cli::run()` and passes its return value to `std::process::exit()`
- **`lib.rs`** -- Library root that re-exports `cli`, `registry`, and `rules` as public modules
- **`cli.rs`** -- Orchestrates the lint pipeline: parses `std::env::args()` for an optional directory path argument (defaults to `"."`), validates the path is a directory, builds a `Registry`, registers default rules (currently `LineCountRule`), uses `WalkDir` to find all `SKILL.md` files from the resolved root, runs each registered rule against each file's content, prints violations to stdout, and returns `0` or `1`
- **`registry.rs`** -- Defines the `Rule` trait (`name`, `description`, `run`) and `Registry` struct that holds `Vec<Box<dyn Rule>>`. Rules return `None` for pass and `Some(message)` for violations
- **`rules/`** -- Submodule containing individual rule implementations; see `@/src/rules/docs.md`

### Things to Know

- The `Rule` trait's `run` method receives the full file content as `&str` and returns `Option<String>` -- `None` means the file passes, `Some` carries the violation message
- `cli::run()` strips the root path prefix from discovered file paths before printing, so output shows relative paths like `subdir/SKILL.md` regardless of whether the root was `"."` or an absolute path argument
- Adding a new rule requires: implementing the `Rule` trait, re-exporting the module in `rules/mod.rs`, and registering it in `cli::run()`
- Invalid directory arguments produce an error message on stderr and exit code 1, before any linting occurs

Created and maintained by Nori.
