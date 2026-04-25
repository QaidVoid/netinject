# `netinject fuzz`

Run fuzzing using **ffuf**. Currently a stub pending full implementation.

## Usage

```bash
netinject fuzz [OPTIONS]
```

## Options

| Flag | Description |
|------|-------------|
| `--target <URL>` | Target URL |
| `--fuzz-point <POINT>` | Fuzz point (parameter name, directory) |
| `--config <PATH>` | Config file path |
| `--auth <NAME>` | Auth profile |
| `--format <FORMAT>` | Output format |
| `--output <PATH>` | Output file |
| `--dry-run` | Show command without executing |

## Configuration

The ffuf adapter is configured in `netinject.toml`:

```toml
[adapters.ffuf]
wordlist = "/usr/share/seclists/Discovery/Web-Content/common.txt"
threads = 40
timeout = 10
recursive = false
```

## Status

This command is currently in development. The adapter trait is defined and the subprocess invocation logic is in place, but output parsing is not yet implemented.
