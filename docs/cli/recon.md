# `netinject recon`

Run HTTP discovery and probing using **httpx**.

## Usage

```bash
netinject recon --target <URL>
```

## What It Does

Invokes `httpx` with the following probes:
- Status code
- Page title
- Web server
- Request method
- Content type
- Host IP
- Technology detection (if enabled in config)

## Options

| Flag | Description |
|------|-------------|
| `--target <URL>` | Target URL (required) |
| `--config <PATH>` | Config file path |
| `--auth <NAME>` | Auth profile |
| `--format <FORMAT>` | Output format |
| `--output <PATH>` | Output file |
| `--dry-run` | Show command without executing |

## Examples

```bash
netinject recon --target https://example.com
netinject recon --target https://example.com --format jsonl
netinject recon --target https://example.com --auth staging
```

## Output

Findings are categorized as `Recon` with `Info` severity, containing:
- URL, status code, page title
- Web server, content type
- Host IP address
- Detected technologies (if enabled)
