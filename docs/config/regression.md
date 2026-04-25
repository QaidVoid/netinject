# Regression Configuration

Regression settings control how netinject detects changes between baselines and current responses.

## Configuration

```toml
[regression]
status_code_change = "breaking"
schema_drift = "breaking"
timing_threshold = 2.0
header_change = "info"
body_hash_change = "warning"
```

## Threshold Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `status_code_change` | `breaking` | Severity when an endpoint returns a different status code |
| `schema_drift` | `breaking` | Severity when the response body schema changes |
| `timing_threshold` | `2.0` | Multiplier for response time anomalies (baseline * threshold = alert) |
| `header_change` | `info` | Severity when response headers are added or removed |
| `body_hash_change` | `warning` | Severity when the response body content hash changes |

## Severity Levels

Each setting maps a type of change to a severity:

| Level | Meaning |
|-------|---------|
| `breaking` | The change likely breaks clients. Treated as high-severity. |
| `warning` | The change is notable but may be intentional. |
| `info` | The change is informational. Logged but not flagged. |

## How Regression Works

1. **Capture a baseline**: `netinject baseline --target <URL>` records status codes, headers, body hashes, schema hashes, and response times for every endpoint.
2. **Check for regressions**: `netinject regress --target <URL>` re-probes the same endpoints and compares responses against the stored baseline.
3. **Report diffs**: Any differences are categorized and assigned severity based on these thresholds.

## Change Types

### Status Code Change

An endpoint that previously returned `200` now returns `404` or `500`. Almost always indicates a regression.

### Schema Drift

The structure of the JSON response has changed (new required fields, removed fields, type changes). Detected by comparing a structural hash of the response body.

### Timing Anomaly

Response time exceeds `baseline_ms * timing_threshold`. A baseline of 100ms with a threshold of 2.0 means alerts fire at 200ms+. Useful for catching performance regressions.

### Header Change

A response header was added or removed. Usually informational (CDN changes, server updates).

### Body Hash Change

The full response body hash differs. Catch-all for any content change not caught by schema analysis.

## Example Configs

Strict (fail on any change):

```toml
[regression]
status_code_change = "breaking"
schema_drift = "breaking"
timing_threshold = 1.5
header_change = "warning"
body_hash_change = "breaking"
```

Relaxed (only flag real breakage):

```toml
[regression]
status_code_change = "breaking"
schema_drift = "warning"
timing_threshold = 3.0
header_change = "info"
body_hash_change = "info"
```
