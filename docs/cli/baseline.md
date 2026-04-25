# `netinject baseline`

Manage baselines for regression detection. Capture, list, and diff baseline snapshots.

## Subcommands

| Command | Description |
|---------|-------------|
| `baseline capture` | Capture a baseline from the live API |
| `baseline list` | List captured baselines |
| `baseline diff <A> <B>` | Compare two baselines |

## Capture a Baseline

```bash
netinject baseline capture --spec openapi.yaml --auth staging
```

This hits every endpoint from the spec and records:
- Status codes
- Response headers
- Response body schema (structural hash)
- Response timing

Stored as a JSON snapshot in `~/.netinject/baselines/`.

## List Baselines

```bash
netinject baseline list
```

## Diff Baselines

```bash
netinject baseline diff baseline-2024-01-15.json baseline-2024-02-01.json
```

## Status

Baseline capture and comparison are in development. The types and storage schema are defined.
