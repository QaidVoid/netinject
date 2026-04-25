# Quick Start

Go from zero to your first security scan in under 5 minutes.

## 1. Initialize a Project

```bash
netinject init --name my-api
```

This creates `netinject.toml` with a full configuration template. Edit it with your target URL:

```toml
[project]
name = "my-api"
target = "https://api.staging.example.com"
```

## 2. Check Your Tools

```bash
netinject check
```

Make sure at least `httpx` and `nuclei` are available.

## 3. Run a Quick Scan

Single-target vulnerability scan:

```bash
netinject scan --target https://api.staging.example.com
```

## 4. Run a Recon

Discover live endpoints and technologies:

```bash
netinject recon --target https://api.staging.example.com
```

## 5. Run a Full Pipeline

Execute a multi-step pipeline (recon then scan):

```bash
netinject run --target https://api.staging.example.com --pipeline full-api-scan
```

Preview what would execute without actually running anything:

```bash
netinject run --target https://api.staging.example.com --dry-run
```

## 6. View Sessions

Every run is recorded as a session:

```bash
netinject sessions list
```

## 7. With an OpenAPI Spec

For spec-driven testing, point to your OpenAPI file:

```bash
netinject scan --target https://api.example.com --spec ./openapi.yaml
```

## Output Formats

All commands support multiple output formats:

```bash
# Terminal table (default)
netinject scan --target https://example.com

# JSON Lines (for piping)
netinject scan --target https://example.com --format jsonl

# Markdown report
netinject scan --target https://example.com --format markdown --output report.md

# SARIF (for GitHub Advanced Security)
netinject scan --target https://example.com --format sarif --output findings.sarif
```
