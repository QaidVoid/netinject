# `netinject run`

Execute a full pipeline. A pipeline is a sequence of adapter runs with dependency resolution and inter-step data passing.

## Usage

```bash
netinject run [OPTIONS]
```

## Options

| Flag | Description |
|------|-------------|
| `--pipeline <NAME>` | Pipeline name from config (default: `full-api-scan`) |
| `--target <URL>` | Target URL (overrides config) |
| `--target-file <PATH>` | File with target URLs (one per line) |
| `--config <PATH>` | Path to config file |
| `--auth <NAME>` | Auth profile to use |
| `--format <FORMAT>` | Output format: `table`, `json`, `jsonl`, `markdown`, `sarif` |
| `--output <PATH>` | Write output to file |
| `--dry-run` | Show what would run without executing |
| `--verbose` | Verbose logging |
| `--quiet` | Suppress progress, show only findings |

## Examples

```bash
# Run default pipeline
netinject run --target https://api.example.com

# Run specific pipeline from config
netinject run --target https://api.example.com --pipeline quick-scan

# Multiple targets from file
netinject run --target-file urls.txt --pipeline full-api-scan

# Preview execution plan without running tools
netinject run --target https://api.example.com --dry-run

# With auth and SARIF output
netinject run --target https://api.example.com --auth staging --format sarif --output results.sarif
```

## How It Works

1. Loads the pipeline definition from config
2. Topologically sorts steps by their `depends_on` fields
3. Executes steps in order, passing URLs between steps
4. Each step's output URLs become input to downstream steps
5. All findings are collected, saved to a session, and output

## Pipeline Data Flow

```
httpx (recon) discovers URLs, which feed into both:
  nuclei (scan) to find vulnerabilities
  ffuf (fuzz) to find hidden paths
```

## Target File Format

One URL per line. Lines starting with `#` are comments. Blank lines are ignored:

```
# Staging endpoints
https://api.staging.example.com/users
https://api.staging.example.com/orders

# Dev endpoints
https://api.dev.example.com/users
```
