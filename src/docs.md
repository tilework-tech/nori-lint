# Noridoc: src

Path: @/src

### Overview

- Contains the binary entry point for the `nori-lint` CLI
- Currently a single file (`main.rs`) with placeholder output

### How it fits into the larger codebase

- `@/src/main.rs` is the sole compilation target -- Cargo compiles it into the `nori-lint` binary
- Integration tests in `@/tests/cli.rs` exercise this binary as a subprocess via `assert_cmd`
- No library crate exists; all executable logic lives here

### Core Implementation

- `fn main()` in `main.rs` prints `"helloworld"` to stdout using `println!`
- No argument parsing, configuration, or external dependencies are used yet

### Things to Know

- The binary currently has no error handling or exit code logic -- it always exits successfully
- As the project grows, core logic should be extracted into a `lib.rs` to enable unit testing independent of the binary subprocess

Created and maintained by Nori.
