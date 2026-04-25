# Adapter Configuration

Each security tool has its own adapter settings. These control how netinvoke invokes the underlying binary.

## ffuf

```toml
[adapters.ffuf]
wordlist = "/usr/share/seclists/Discovery/Web-Content/common.txt"
threads = 40
timeout = 10
recursive = false
```

| Setting | Default | Description |
|---------|---------|-------------|
| `wordlist` | `/usr/share/seclists/Discovery/Web-Content/common.txt` | Path to fuzzing wordlist |
| `threads` | 40 | Number of concurrent threads |
| `timeout` | 10 | HTTP timeout in seconds |
| `recursive` | false | Enable recursive directory fuzzing |

## nuclei

```toml
[adapters.nuclei]
templates = []
severity = []
rate_limit = 100
```

| Setting | Default | Description |
|---------|---------|-------------|
| `templates` | `[]` (all) | Template tags or paths to run |
| `severity` | `[]` (all) | Filter by severity: `critical`, `high`, `medium`, `low`, `info` |
| `rate_limit` | 100 | Maximum requests per second |

## httpx

```toml
[adapters.httpx]
threads = 40
rate_limit = 150
tech_detect = false
```

| Setting | Default | Description |
|---------|---------|-------------|
| `threads` | 40 | Number of concurrent threads |
| `rate_limit` | 150 | Maximum requests per second |
| `tech_detect` | false | Enable technology detection probes |

## sqlmap

```toml
[adapters.sqlmap]
level = 3
risk = 2
batch = true
```

| Setting | Default | Description |
|---------|---------|-------------|
| `level` | 3 | Detection level (1-5, higher = more tests) |
| `risk` | 2 | Risk of tests (1-3, higher = more aggressive) |
| `batch` | true | Never prompt for user input |

## mitmproxy

```toml
[adapters.mitmproxy]
listen_host = "127.0.0.1"
listen_port = 8080
upstream_proxy = ""
```

| Setting | Default | Description |
|---------|---------|-------------|
| `listen_host` | `127.0.0.1` | Proxy listen address |
| `listen_port` | 8080 | Proxy listen port |
| `upstream_proxy` | (empty) | Upstream proxy URL for chained proxying |

## How Adapters Work

Adapters invoke external tools as subprocesses. netinject does not re-implement any scanning, fuzzing, or proxying logic. Each adapter:

1. Builds a command line from config + input URLs
2. Runs the tool as a subprocess
3. Parses the tool's JSONL output
4. Normalizes results into unified `Finding` structs

All tools must be installed separately. Run `netinject check` to verify which tools are available.
