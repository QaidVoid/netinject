# `netinject init`

Initialize a new netinject project by creating a default `netinject.toml` config file.

## Usage

```bash
netinject init
```

## What It Does

Creates a `netinject.toml` in the current directory with sensible defaults:

```toml
[general]
scope_include = ["https://api.example.com/*"]
scope_exclude = ["/health", "/metrics"]
default_auth = "default"

[adapters.httpx]
threads = 25
timeout = 10

[adapters.nuclei]
templates = "cves,vulnerabilities,exposures"
severity = "medium,high,critical"
timeout = 180

[adapters.ffuf]
wordlist = "/usr/share/seclists/Discovery/Web-Content/common.txt"
threads = 40
timeout = 10

[auth.default]
type = "bearer"
token = "$BEARER_TOKEN"
```

## Examples

```bash
# Initialize in current directory
netinject init

# Then edit the generated config
$EDITOR netinject.toml
```
