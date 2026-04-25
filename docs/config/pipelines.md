# Pipelines

Pipelines define multi-step execution plans. Each step runs an adapter, and steps can depend on the output of previous steps.

## Definition

Pipelines are defined in `netinject.toml`:

```toml
[[pipeline]]
name = "full-api-scan"
description = "Recon, scan, and fuzz"

[[pipeline.steps]]
adapter = "httpx"
label = "recon"

[[pipeline.steps]]
adapter = "nuclei"
label = "scan"
depends_on = "recon"

[[pipeline.steps]]
adapter = "ffuf"
label = "fuzz"
depends_on = "recon"
```

## Step Fields

| Field | Required | Description |
|-------|----------|-------------|
| `adapter` | Yes | Adapter name: `httpx`, `nuclei`, `ffuf`, `sqlmap`, `mitmproxy` |
| `label` | No | Step identifier. Used by `depends_on` references. Defaults to `step-N`. |
| `depends_on` | No | Label of a step that must complete before this one runs. |
| `config` | No | Custom config override key (reserved for future use). |

## Dependency Resolution

Steps are sorted topologically based on `depends_on`:

1. Steps with no dependencies run first (using the initial target URLs).
2. Steps with dependencies wait for their parent step to complete.
3. The parent step's output URLs become the input for the dependent step.
4. Cycles are detected and reported as errors.

## Data Flow

```
httpx (recon) discovers 50 URLs
  ├── nuclei (scan) tests those 50 URLs for vulnerabilities
  └── ffuf (fuzz) fuzzes those 50 URLs for hidden paths
```

The URLs found by each step are extracted from that step's findings and passed to downstream steps.

## Running a Pipeline

```bash
# Run the default pipeline
netinject run --target https://api.example.com

# Run a named pipeline
netinject run --target https://api.example.com --pipeline quick-scan

# Preview without executing
netinject run --target https://api.example.com --dry-run
```

## Multiple Pipelines

You can define multiple pipelines in one config:

```toml
[[pipeline]]
name = "quick-scan"
description = "Fast recon + scan"

[[pipeline.steps]]
adapter = "httpx"
label = "recon"

[[pipeline.steps]]
adapter = "nuclei"
label = "scan"
depends_on = "recon"

[[pipeline]]
name = "deep-scan"
description = "Full recon, scan, fuzz, and SQLi test"

[[pipeline.steps]]
adapter = "httpx"
label = "recon"

[[pipeline.steps]]
adapter = "nuclei"
label = "scan"
depends_on = "recon"

[[pipeline.steps]]
adapter = "ffuf"
label = "fuzz"
depends_on = "recon"

[[pipeline.steps]]
adapter = "sqlmap"
label = "sqli"
depends_on = "scan"
```

Then select with `--pipeline <name>`.
