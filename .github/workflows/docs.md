# Noridoc: workflows

Path: @/.github/workflows

### Overview

- GitHub Actions CI pipeline for the nori-lint project
- Enforces code quality gates on every push and pull request

### How it fits into the larger codebase

- Acts as the automated quality gate for all code entering the repository -- no code merges without passing formatting, linting, and tests
- Runs `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test` against the Rust codebase in `@/src/` and `@/tests/`
- Uses `Swatinem/rust-cache@v2` on the clippy and build-and-test jobs to cache compiled dependencies between runs

### Core Implementation

- **`ci.yml`** defines three parallel jobs:

| Job | Purpose | Key command |
|---|---|---|
| `fmt` | Enforces consistent code formatting | `cargo fmt --check` |
| `clippy` | Static analysis with all warnings treated as errors | `cargo clippy -- -D warnings` |
| `build-and-test` | Compiles the project and runs all tests | `cargo build && cargo test` |

- All jobs run on `ubuntu-latest` with the stable Rust toolchain via `dtolnay/rust-toolchain@stable`
- The `fmt` job additionally installs the `rustfmt` component; the `clippy` job installs the `clippy` component

### Things to Know

- The workflow triggers on pushes to `main` and on all pull requests; a concurrency group cancels superseded runs on the same branch
- The `fmt` job does not use dependency caching since `cargo fmt --check` does not compile code
- `cargo clippy -- -D warnings` means any Clippy warning fails the build, not just errors

Created and maintained by Nori.
