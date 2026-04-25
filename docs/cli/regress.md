# `netinject regress`

Check the current state of an API against a previously captured baseline.

## Usage

```bash
netinject regress check [OPTIONS]
```

## Options

| Flag | Description |
|------|-------------|
| `--against <BASELINE>` | Baseline to check against (default: latest) |
| `--paths <PATTERNS>` | Only check specific endpoint paths (comma-separated) |
| `--config <PATH>` | Config file path |
| `--auth <NAME>` | Auth profile |
| `--format <FORMAT>` | Output format |

## Examples

```bash
# Check against latest baseline
netinject regress check --auth staging

# Check against a specific baseline
netinject regress check --against baseline-2024-01-15.json

# Only check specific paths
netinject regress check --paths "/api/users/*,/api/orders/*"
```

## What It Detects

The regression engine compares current responses against the baseline and classifies changes:

| Change Type | Default Severity |
|-------------|-----------------|
| Status code changed | Breaking |
| Schema drift (field added/removed/type changed) | Breaking |
| Body hash changed | Warning |
| Timing anomaly (over 2x baseline) | Warning |
| Header changes | Info |

Severity thresholds are configurable in `netinject.toml`.

## Status

Regression checking is in development. The types, severity classification, and storage schema are defined.
