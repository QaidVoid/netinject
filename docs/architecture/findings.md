# Findings

All tool output is normalized into a single `Finding` struct, regardless of which adapter produced it.

## Structure

```rust
pub struct Finding {
    pub id: String,
    pub session_id: String,
    pub source: String,          // adapter name (httpx, nuclei, ffuf, etc.)
    pub category: Category,      // Recon, Vulnerability, Fuzz, Injection, Info
    pub severity: Severity,      // Info, Low, Medium, High, Critical
    pub title: String,
    pub description: String,
    pub url: String,
    pub evidence: String,
    pub raw_output: serde_json::Value,
    pub timestamp: String,
}
```

## Severity Levels

| Level | Numeric | Typical Sources |
|-------|---------|-----------------|
| Critical | 4 | SQLi, RCE, auth bypass |
| High | 3 | XSS, SSRF, IDOR |
| Medium | 2 | Misconfig, info leak |
| Low | 1 | Missing headers, verbose errors |
| Info | 0 | Recon results, metadata |

## Categories

| Category | Purpose |
|----------|---------|
| `Recon` | URL discovery, tech detection (httpx) |
| `Vulnerability` | Confirmed vulns (nuclei) |
| `Fuzz` | Fuzzing hits (ffuf) |
| `Injection` | SQL injection findings (sqlmap) |
| `Info` | General information, metadata |

## Source-Specific Fields

Each adapter maps its tool-specific JSONL fields into the common structure:

- **httpx**: `url` -> `url`, `tech` -> `evidence`, `status_code` -> `description`
- **nuclei**: `matched` -> `url`, `template_id` -> `title`, `info` -> `description`
- **ffuf**: `url` -> `url`, `input` -> `evidence`, `status`/`length` -> `description`

The original raw JSONL is always preserved in `raw_output` for detailed analysis.
