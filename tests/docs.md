# Noridoc: tests

Path: @/tests

### Overview

- Integration tests for the `nori-lint` binary
- Tests invoke the compiled binary as a subprocess and assert on its output and exit status

### How it fits into the larger codebase

- Exercises the compiled binary produced from `@/src/main.rs`
- Run via `cargo test` -- Cargo automatically discovers files in the `tests/` directory as integration test targets
- Each file in this directory is compiled as a separate test crate with access only to the public interface (the binary itself)

### Core Implementation

- `cli.rs` contains a single test (`prints_helloworld`) that uses the `assert_cmd` crate's `cargo_bin_cmd!` macro to locate and run the `nori-lint` binary
- The test asserts: successful exit (exit code 0) and stdout contains `"helloworld\n"`

### Things to Know

- `cargo_bin_cmd!("nori-lint")` resolves the binary path using the crate name from `@/Cargo.toml` -- if the package name changes, this macro call must be updated
- `assert_cmd` compiles the binary in debug mode during `cargo test`, so the binary must compile successfully for tests to run at all
- Tests run the binary as a child process, meaning they test the full end-to-end behavior including stdout formatting and exit codes

Created and maintained by Nori.
