# AGENTS.md

Guidelines for AI agents working on the netinject project.

## Project Overview

netinject is a lightweight Rust CLI tool that orchestrates existing API security testing tools (ffuf, nuclei, httpx, sqlmap, mitmproxy) into unified pipelines. It parses OpenAPI specs for intelligent testing, captures baselines, and detects API regressions. It does NOT reimplement scanning, fuzzing, or proxying — it wraps existing tools and adds value on top (session management, regression tracking, unified output).

## Development Workflow

### Commits

- **Commit in small chunks** — One logical change per commit
- **Never commit broken state** — All code must compile and pass tests
- **Format before commit** — Run `cargo fmt` before every commit
- **Fix clippy issues** — Run `cargo clippy` and address all warnings before committing

### Commit Messages

Follow conventional commit format with imperative mood:

```
type: message
```

Types:
- `feat:` — New feature
- `fix:` — Bug fix
- `refactor:` — Code refactoring
- `test:` — Adding or updating tests
- `docs:` — Documentation changes
- `chore:` — Maintenance tasks
- `perf:` — Performance improvements
- `style:` — Code style changes (formatting, etc.)
- `ci:` — CI/CD configuration changes

Examples:
- `feat: add nuclei adapter with JSONL output parsing`
- `fix: handle missing tool binaries gracefully in check command`
- `refactor: extract scope matching into reusable module`

## Code Quality

### Formatting

```bash
cargo fmt
```

Always run before committing.

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

All clippy warnings must be addressed before committing.

### Testing

```bash
cargo nextest --all-features
# or
cargo test --all-features
```

All tests must pass before committing.

### Nix

This project uses a Nix flake for reproducible builds and dev environment.

```bash
# Enter dev shell (provides Rust toolchain + security tools)
nix develop

# Build
nix build

# Run checks (clippy, fmt, tests)
nix flake check
```

## Pre-commit Checklist

Before every commit, ensure:

1. [ ] `cargo fmt` — Code is formatted
2. [ ] `cargo clippy` — No warnings
3. [ ] `cargo test` — All tests pass
4. [ ] `cargo build` — Clean build with no errors

## Project Structure

```
netinject/
├── Cargo.toml
├── flake.nix              # Nix flake (build + dev shell + checks)
├── src/
│   ├── main.rs            # CLI entry point (clap)
│   ├── cli/
│   │   ├── args.rs        # CLI argument definitions
│   │   └── commands/      # One module per subcommand
│   ├── config/            # Config loading & merging (TOML)
│   ├── spec/              # OpenAPI/Swagger spec parsing
│   ├── adapters/          # Tool adapters (ffuf, nuclei, httpx, sqlmap, mitmproxy)
│   ├── pipeline/          # Pipeline definition & execution engine
│   ├── session/           # Session storage (SQLite)
│   ├── baseline/          # Baseline capture & regression detection
│   ├── finding/           # Normalized finding types
│   ├── report/            # Output formatters (JSONL, Markdown, SARIF, table)
│   ├── auth/              # Auth profile resolution & injection
│   ├── scope/             # URL scope matching (include/exclude)
│   └── util/              # Subprocess management, HTTP helpers
├── tests/
│   ├── integration/       # Integration tests per adapter
│   └── fixtures/          # Sample OpenAPI specs, configs
└── benches/               # Benchmarks
```

## Design Principles

- **Orchestrator, not implementor** — wrap existing tools, don't reimplement them
- **CLI-first** — everything scriptable, automatable, pipeable
- **Zero runtime deps** �� single static binary (tools it orchestrates are external)
- **Offline-first** — all data local (SQLite), no cloud dependency
- **API-aware** — understands OpenAPI specs for intelligent, targeted testing
- **Async** — tokio-based for concurrent tool execution and HTTP operations
- **Modular** — each adapter, command, and module is independent and testable in isolation

## Dependencies

Core dependencies are chosen to be well-maintained and idiomatic:

| Purpose | Crate | Notes |
|---------|-------|-------|
| Async runtime | `tokio` | Industry standard |
| CLI | `clap` (derive) | De facto Rust CLI framework |
| HTTP client | `ureq` | For baseline capture, replay, spec fetching |
| SQLite | `rusqlite` (bundled) | Embedded, zero-config |
| OpenAPI | `oas3` | OpenAPI 3.x parsing |
| YAML | `saphyr` | YAML parsing (spec files, configs) |
| Serialization | `serde` + `serde_json` + `toml` | Everywhere |
| Errors | `anyhow` (app) + `thiserror` (lib) | Ergonomic error handling |
| Terminal | `tabled`, `indicatif`, `console` | Tables, progress, colors |

Avoid adding dependencies for things that can be implemented in a few dozen lines.

## Additional Notes

- Edition: Rust 2024
- Use `thiserror` for library error types, `anyhow` for application-level errors
- Adapter trait (`adapters/mod.rs`) is the core extension point — all tools implement it
- Security tools are invoked as subprocesses — never reimplement their functionality
- All tool output is normalized into `Finding` structs regardless of source
