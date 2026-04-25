# Reports

Findings can be exported in multiple formats. All formats contain the same data, just rendered differently.

## Formats

### Table (Default)

Human-readable terminal output using `tabled`.

```bash
netinject scan --target https://api.example.com
netinject report <session-id> --format table
```

Columns: Severity, Source, Title, URL

### JSONL

One JSON object per line. Useful for piping to `jq` or other tools.

```bash
netinject scan --target https://api.example.com --output jsonl
netinject report <session-id> --format jsonl
```

Each line is a complete `Finding` object.

### JSON

Pretty-printed JSON array.

```bash
netinject report <session-id> --format json
```

### Markdown

Structured markdown report with severity grouping.

```bash
netinject report <session-id> --format markdown
```

Sections: Summary by severity, then detailed findings grouped by severity (Critical first).

### SARIF

Static Analysis Results Interchange Format. For CI/CD integration with GitHub Code Scanning, Azure DevOps, etc.

```bash
netinject report <session-id> --format sarif
```

Follows the [SARIF v2.1.0](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html) specification. Rules are keyed by adapter name and category.

## Output Destination

```bash
# Stdout (default)
netinject report <session-id> --format json

# File
netinject report <session-id> --format sarif --output-file results.sarif
```

## CI/CD Integration

The SARIF format integrates with GitHub Advanced Security:

```yaml
- name: Run API security scan
  run: netinject run --target $API_URL --session ci-scan

- name: Upload SARIF results
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```
