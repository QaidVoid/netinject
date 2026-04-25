# Adapters

Adapters wrap external security tools. Each adapter implements the `Adapter` trait and handles subprocess execution, argument construction, and output parsing.

## Adapter Trait

```rust
#[async_trait]
pub trait Adapter {
    fn name(&self) -> &str;
    async fn run(&self, input: AdapterInput) -> Result<Vec<Finding>>;
}
```

Every adapter receives an `AdapterInput` (target URLs, auth headers, scope rules) and returns normalized `Finding` structs.

## Supported Adapters

### httpx (Recon)

Probes URLs and collects metadata: status codes, titles, technologies, content lengths.

```bash
netinject recon --target https://api.example.com
```

Flags:
- `--threads` — concurrent probes (default: 25)
- `--follow-redirects` — follow HTTP redirects
- `--tech-detect` — detect web technologies

### nuclei (Scan)

Runs vulnerability templates against targets. Outputs CVE IDs, severity, and matched URLs.

```bash
netinject scan --target https://api.example.com
```

Flags:
- `--templates` — template directory or tag filter
- `--severity` — minimum severity filter (low, medium, high, critical)
- `--rate-limit` — requests per second

### ffuf (Fuzz)

Fuzzes URL paths, parameters, and headers. Uses wordlists for discovery.

```bash
netinject fuzz --target https://api.example.com/FUZZ
```

Flags:
- `--wordlist` — path to wordlist file
- `--method` — HTTP method (default: GET)
- `--filter-code` — comma-separated status codes to exclude
- `--threads` — concurrent connections (default: 40)

### sqlmap (SQLi)

Tests for SQL injection vulnerabilities. Triggered via pipeline or directly.

### mitmproxy (Proxy)

Captures and replays traffic through an intercepting proxy. Used for manual testing workflows.

## Adding a New Adapter

1. Create `src/adapters/<name>.rs`
2. Implement the `Adapter` trait
3. Register in `src/adapters/mod.rs`
4. Add CLI flags in `src/commands/<name>.rs`
5. Add tests in `tests/integration/`

Each adapter must:
- Build the subprocess command from `AdapterInput`
- Parse the tool's JSONL output into `Finding` structs
- Handle missing binaries gracefully (checked by `netinject check`)
- Respect scope rules (include/exclude URL patterns)
