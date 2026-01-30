# Noridoc: nori-lint

Path: @/

### Overview

- Rust binary crate for the `nori-lint` command-line tool
- Currently in initial scaffolding phase -- the binary prints "helloworld" as a placeholder
- Uses Rust edition 2024 and follows the standard Cargo binary crate layout (`src/main.rs` for the binary, `tests/` for integration tests)

### How it fits into the larger codebase

- Part of the Nori monorepo at `@/../nori/`; sits alongside other Nori projects like `nori-watchtower`, `nori-toolbox`, `nori-premortem`, etc.
- Intended to become a standalone linting CLI tool -- currently no integration points with other Nori projects

### Core Implementation

- **Entry point:** `@/src/main.rs` -- contains `fn main()` which prints `"helloworld"` to stdout
- **Integration tests:** `@/tests/cli.rs` -- uses the `assert_cmd` crate to invoke the compiled `nori-lint` binary as a subprocess and assert on its stdout output
- **Build configuration:** `@/Cargo.toml` declares the package name `nori-lint` with `assert_cmd` as a dev-dependency for testing

### Things to Know

- The `assert_cmd` crate's `cargo_bin_cmd!` macro locates the compiled binary by crate name, so renaming the package in `Cargo.toml` requires updating the macro argument in test files
- There is no library crate (`lib.rs`) -- all code is in the binary target. If the project grows, extracting logic into a library crate will allow unit testing without subprocess overhead

Created and maintained by Nori.
