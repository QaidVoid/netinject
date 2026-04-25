# `netinject sessions`

List stored sessions. Sessions track which tests ran against which targets, when, and what they found.

## Usage

```bash
netinject sessions list
```

## Output

```
 ID   Created              Target                 Auth      Status    Findings
 1    2024-01-15 10:30:00  https://api.example.com staging   completed 15
 2    2024-01-16 14:20:00  https://staging.example.com staging   completed 3
```

## Details

Sessions are stored in SQLite at `~/.netinject/sessions.db`. Each session records:
- Target URL
- Auth profile used
- Pipeline or command that ran
- Start and end timestamps
- Number of findings by severity

Use session IDs as input to the `report` command to generate reports for a specific session.

## Examples

```bash
# List all sessions
netinject sessions list

# Filter (coming soon)
netinject sessions list --target example.com
```
