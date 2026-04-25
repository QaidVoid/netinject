# Session Store

Every netinject run is recorded as a session in a local SQLite database. This enables regression tracking, historical comparison, and result replay.

## Location

The database is stored at `.netinject/sessions.db` relative to the project root (created by `netinject init`).

## Schema

### Sessions Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | UUID (primary key) |
| `name` | TEXT | Optional user-provided name |
| `command` | TEXT | Subcommand that created the session |
| `target` | TEXT | Primary target URL or file |
| `created_at` | TEXT | ISO 8601 timestamp |
| `status` | TEXT | `running`, `completed`, `failed` |

### Findings Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | UUID (primary key) |
| `session_id` | TEXT | Foreign key to sessions |
| `source` | TEXT | Adapter name |
| `category` | TEXT | Finding category |
| `severity` | TEXT | Severity level |
| `title` | TEXT | Short description |
| `description` | TEXT | Detailed description |
| `url` | TEXT | Affected URL |
| `evidence` | TEXT | Proof or reproduction details |
| `raw_output` | TEXT | Original JSONL line |
| `timestamp` | TEXT | ISO 8601 timestamp |

## Session Lifecycle

1. **Create**: A new session is created when any command runs with `--session`
2. **Record**: Findings are inserted as adapters produce them
3. **Complete**: Session status is updated to `completed` or `failed`
4. **Query**: `netinject sessions` lists past runs
5. **Replay**: `netinject replay <session-id>` re-displays findings

## Session Naming

```bash
# Auto-generated name
netinject scan --target https://api.example.com

# Explicit name
netinject scan --target https://api.example.com --session weekly-scan

# List sessions
netinject sessions

# Show findings from a specific session
netinject replay <session-id>
```

## Data Retention

Sessions persist until manually deleted. There is no automatic cleanup. Use SQLite directly to prune old sessions if needed.
