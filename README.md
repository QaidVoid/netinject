# netinject

Lightweight CLI orchestrator for API security testing pipelines.

Wraps existing tools (httpx, nuclei, ffuf, sqlmap, mitmproxy) into unified pipelines with session tracking, baseline capture, and regression detection.

## Quick Start

```bash
# Build
cargo build --release

# Initialize a project
netinject init --name my-api

# Edit the generated netinject.toml with your target
# Then run:
netinject check     # verify tool installations
netinject recon     # discover endpoints (httpx)
netinject scan      # vulnerability scan (nuclei)
netinject fuzz      # fuzz endpoints (ffuf)
netinject run       # full pipeline (recon → fuzz → scan)
```

Config is auto-discovered from `netinject.toml` in the current directory or any parent.

## What It Does

- **Orchestrates** security tools into ordered pipelines with dependency resolution
- **Parses** OpenAPI 3.x specs for targeted, spec-driven testing
- **Normalizes** all tool output into a unified `Finding` format
- **Tracks** every run as a session in a local SQLite database
- **Captures** API baselines and detects regressions between runs
- **Outputs** findings as tables, JSONL, Markdown, or SARIF

## What It Doesn't Do

Reimplement scanning, fuzzing, or proxying. It calls the real tools and adds orchestration on top.

## Supported Tools

| Tool | Purpose | Install |
|------|---------|---------|
| httpx | HTTP probing and recon | `go install github.com/projectdiscovery/httpx/cmd/httpx@latest` |
| nuclei | Vulnerability scanning | `go install github.com/projectdiscovery/nuclei/v3/cmd/nuclei@latest` |
| ffuf | Fuzzing | `go install github.com/ffuf/ffuf/v2@latest` |
| sqlmap | SQL injection | `pip install sqlmap` |
| mitmproxy | MITM traffic capture | `pip install mitmproxy` |

## Commands

| Command | Description |
|---------|-------------|
| `init` | Create `netinject.toml` config template |
| `check` | Verify tool installations |
| `recon` | Discover endpoints (httpx) |
| `scan` | Vulnerability scan (nuclei) |
| `fuzz` | Fuzz endpoints (ffuf) |
| `run` | Execute a multi-step pipeline |
| `sessions` | Browse and compare past runs |
| `baseline` | Capture and diff API baselines |
| `regress` | Detect regressions against a baseline |
| `replay` | Replay captured requests |
| `completions` | Generate shell completions |

## Config

```toml
[project]
name = "my-api"
target = "https://api.staging.example.com"
spec = "./openapi.yaml"

[scope]
include = ["https://api.staging.example.com/*"]
exclude = ["https://api.staging.example.com/admin/*"]

[adapters.nuclei]
severity = ["high", "critical"]
rate_limit = 100

[[auth]]
name = "staging"
type = "bearer"
token = "${STAGING_TOKEN}"

[[pipeline]]
name = "full-api-scan"
description = "Full API security scan"
steps = [
  { adapter = "httpx", label = "recon" },
  { adapter = "ffuf", label = "fuzz", depends_on = "recon" },
  { adapter = "nuclei", label = "scan", depends_on = "recon" },
]
```

## Output Formats

```bash
netinject scan                        # terminal table (default)
netinject scan --format jsonl         # JSON Lines
netinject scan --format markdown -o report.md
netinject scan --format sarif -o findings.sarif
```

## Building

```bash
cargo build --release
```

Or with Nix:

```bash
nix build
nix develop   # dev shell with Rust toolchain + security tools
```

## Documentation

Full docs at [netinject.qaidvoid.dev](https://netinject.qaidvoid.dev) or build locally:

```bash
cd docs && bun install && bun run docs:dev
```

## License

MIT OR Apache-2.0
