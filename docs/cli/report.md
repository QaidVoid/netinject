# `netinject report`

Generate reports from stored session data. Supports table, JSONL, Markdown, and SARIF formats.

## Usage

```bash
netinject report --session <ID> --format <FORMAT>
```

## Options

| Flag | Description |
|------|-------------|
| `--session <ID>` | Session ID to report on |
| `--format <FORMAT>` | Output format: `table`, `jsonl`, `markdown`, `sarif` |
| `--output <PATH>` | Write to file instead of stdout |
| `--severity <LEVEL>` | Filter by minimum severity |

## Formats

### Table (default)

Human-readable output in the terminal with aligned columns.

### JSONL

One JSON object per finding. Useful for piping into `jq` or other tools.

```bash
netinject report --session 1 --format jsonl | jq '.severity'
```

### Markdown

Markdown table suitable for pasting into issues, PRs, or wikis.

```bash
netinject report --session 1 --format markdown > report.md
```

### SARIF

Static Analysis Results Interchange Format. Used by GitHub Code Scanning and other security platforms.

```bash
netinject report --session 1 --format sarif > results.sarif
```

## Examples

```bash
# Print findings to terminal
netinject report --session 1

# Generate SARIF for GitHub
netinject report --session 1 --format sarif --output results.sarif

# Only show critical and high findings
netinject report --session 1 --severity high
```
