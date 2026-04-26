# Configuration

netinject uses a single TOML config file (`netinject.toml`) to control all behavior: targets, scope, adapter settings, auth profiles, pipelines, and regression thresholds.

## Generating a Config

```bash
netinject init
```

Creates a `netinject.toml` in the current directory with sensible defaults.

## Config Precedence

Settings are resolved in this order, with later sources overriding earlier ones:

1. **Defaults** built into netinject
2. **Auto-discovered config** (`netinject.toml` found by walking up from the current directory)
3. **Explicit config** (`--config path/to/custom.toml`)
4. **CLI flags** (`--target`, `--auth`, etc.)

### Auto-Discovery

If no `--config` flag is provided, netinject automatically looks for `netinject.toml` in the current directory and its parent directories (similar to `.gitignore`). This means you can run commands from any subdirectory of your project:

```bash
netinject init            # creates netinject.toml in current dir
netinject check           # auto-discovers netinject.toml
netinject scan            # reads target from discovered config
netinject run             # runs full-api-scan pipeline from config
```

## Config Structure

```toml
[project]
name = "my-api"
target = "https://api.example.com"
spec = "openapi.yaml"

[scope]
include = ["https://api.example.com/*"]
exclude = ["https://api.example.com/admin/*"]
max_rate = 50
max_concurrent = 10

[adapters.ffuf]
wordlist = "/usr/share/seclists/Discovery/Web-Content/common.txt"
threads = 40
timeout = 10
recursive = false

[adapters.nuclei]
templates = []
severity = []
rate_limit = 100

[adapters.httpx]
threads = 40
rate_limit = 150
tech_detect = false

[adapters.sqlmap]
level = 3
risk = 2
batch = true

[adapters.mitmproxy]
listen_host = "127.0.0.1"
listen_port = 8080
upstream_proxy = ""

[[auth]]
name = "staging"
type = "bearer"
token = "${STAGING_TOKEN}"

[[pipeline]]
name = "full-api-scan"
description = "Recon, scan, and fuzz in sequence"
  [[pipeline.steps]]
  adapter = "httpx"
  label = "recon"

  [[pipeline.steps]]
  adapter = "nuclei"
  label = "scan"
  depends_on = "recon"

[regression]
status_code_change = "breaking"
schema_drift = "breaking"
timing_threshold = 2.0
header_change = "info"
body_hash_change = "warning"
```

## Sections

- [Auth Profiles](/config/auth) for authentication configuration
- [Scope Rules](/config/scope) for URL include/exclude patterns
- [Adapter Config](/config/adapters) for per-tool settings
- [Pipelines](/config/pipelines) for multi-step execution plans
- [Regression](/config/regression) for baseline diff thresholds
