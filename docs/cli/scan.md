# `netinject scan`

Run vulnerability scanning using **nuclei**.

## Usage

```bash
netinject scan --target <URL>
```

## What It Does

Invokes `nuclei` with JSONL output mode and parses every finding into a normalized `Finding` struct. Supports template filtering, severity filtering, and custom headers.

## Options

| Flag | Description |
|------|-------------|
| `--target <URL>` | Target URL (required) |
| `--spec <PATH>` | OpenAPI spec for targeted testing |
| `--config <PATH>` | Config file path |
| `--auth <NAME>` | Auth profile |
| `--format <FORMAT>` | Output format |
| `--output <PATH>` | Output file |
| `--dry-run` | Show command without executing |

## Examples

```bash
# Basic scan
netinject scan --target https://example.com

# With custom severity filter (configured in netinject.toml)
netinject scan --target https://example.com --config netinject.toml

# JSONL output for piping into jq
netinject scan --target https://example.com --format jsonl

# With auth
netinject scan --target https://api.example.com --auth staging

# Dry run
netinject scan --target https://example.com --dry-run
```

## Output

Findings include severity, category, CWE ID, references, and extracted evidence. Categories are auto-detected from the template ID:
- Templates with `sqli`, `inject`, `xss`, or `rce` are categorized as **Injection**
- Templates with `auth`, `login`, or `bypass` are categorized as **Auth**
- Templates with `expos` or `leak` are categorized as **Data Exposure**
- Templates with `misconfig` are categorized as **Misconfiguration**
