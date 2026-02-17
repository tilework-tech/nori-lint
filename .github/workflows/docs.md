# Noridoc: workflows

Path: @/.github/workflows

### Overview

- GitHub Actions CI pipeline for the nori-lint project
- Enforces code quality gates on every push and pull request

### How it fits into the larger codebase

- Acts as the automated quality gate for all code entering the repository
- Runs against the TypeScript codebase in `@/src/` and `@/tests/`
- Uses `actions/setup-node@v4` with Node 22 and npm caching

### Core Implementation

- **`ci.yml`** defines three parallel jobs:

| Job | Purpose | Key command |
|---|---|---|
| `lint` | ESLint, Prettier, and TypeScript type checking | `npm run lint` (runs `eslint . --max-warnings=0`, `prettier . --check`, `tsc --noEmit`) |
| `test` | Runs all vitest tests | `npm test` |
| `build` | Compiles TypeScript to JavaScript | `npm run build` |

- All jobs run on `ubuntu-latest` with Node 22 via `actions/setup-node@v4`
- Dependencies are installed via `npm ci` with npm caching enabled

### Things to Know

- The workflow triggers on pushes to `main` and on all pull requests; a concurrency group cancels superseded runs on the same branch
- The `lint` job runs ESLint, Prettier, and TypeScript type checking via a single `npm run lint` command that uses `concurrently` to run all three in parallel

Created and maintained by Nori.
