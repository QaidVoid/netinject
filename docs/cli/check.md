# `netinject check`

Check that all required external tools are installed and accessible.

## Usage

```bash
netinject check
```

## What It Does

Verifies that the following tools are available on `$PATH`:

| Tool | Purpose | Required |
|------|---------|----------|
| `httpx` | Probing and technology detection | Yes |
| `nuclei` | Vulnerability scanning | Yes |
| `ffuf` | Fuzzing | No |

For each tool, it checks:
- Binary exists on `$PATH`
- Version can be extracted
- Minimum version requirement is met (if applicable)

## Output

```
  httpx   v1.6.0    ✓
  nuclei  v3.2.0    ✓
  ffuf    v2.1.0    ✓ (optional)
  sqlmap  ---       ✗ (optional, not found)
```

## Examples

```bash
# Check all tools
netinject check
```
