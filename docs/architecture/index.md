# Architecture

netinject is an orchestrator, not a scanner. It wraps existing security tools, normalizes their output, and adds session management, regression tracking, and unified reporting on top.

## Core Design Principles

- **Orchestrator, not implementor**: All scanning, fuzzing, and proxying is delegated to external tools via subprocesses.
- **CLI-first**: Everything is scriptable, automatable, and pipeable.
- **Offline-first**: All data is stored locally in SQLite. No cloud dependency.
- **API-aware**: OpenAPI spec parsing enables intelligent, targeted testing.
- **Async**: Built on tokio for concurrent tool execution and HTTP operations.

## Module Overview

```
src/
├── cli/          Argument parsing (clap) and command handlers
├── config/       TOML config loading, merging, and defaults
├── adapters/     Tool wrappers implementing the Adapter trait
├── pipeline/     Multi-step execution with dependency resolution
├── session/      SQLite-backed session and finding storage
├── baseline/     Response capture and regression detection
├── finding/      Normalized Finding type shared across all modules
├── report/       Output formatters (table, JSON, JSONL, Markdown, SARIF)
├── auth/         Auth profile resolution and header injection
├── scope/        URL scope matching with include/exclude globs
├── spec/         OpenAPI 3.x spec parsing
└── types/        Shared types (Severity, Category)
```

## Data Flow

```
CLI args
  └── Config resolution (defaults + file + flags)
      └── Auth profile → HTTP headers
      └── Scope checker → URL filtering
          └── Pipeline execution
              └── Adapter subprocess (ffuf, nuclei, httpx, etc.)
                  └── JSONL output → normalized Findings
                      └── Session store (SQLite)
                      └── Report output (table, JSON, SARIF, etc.)
```

## Key Abstractions

### Adapter Trait

Every tool implements the `Adapter` trait. See [Adapters](/architecture/adapters).

### Finding

All tool output is normalized into a single `Finding` struct. See [Findings](/architecture/findings).

### Session Store

Every run is recorded in SQLite with full finding history. See [Session Store](/architecture/sessions).

### Reports

Findings can be exported in multiple formats. See [Reports](/architecture/reports).
