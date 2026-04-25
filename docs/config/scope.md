# Scope Rules

Scope rules control which URLs netinject is allowed to test. They prevent accidental scanning of production or out-of-scope endpoints.

## Configuration

```toml
[scope]
include = ["https://api.staging.example.com/*"]
exclude = ["https://api.staging.example.com/admin/*"]
max_rate = 50
max_concurrent = 10
```

## Include Patterns

Only URLs matching at least one include pattern are tested. If no include patterns are defined, all URLs are considered in scope.

Patterns use glob syntax:

| Pattern | Matches |
|---------|---------|
| `https://api.example.com/*` | Any single path segment |
| `https://api.example.com/**` | Any path depth |
| `https://*.example.com/*` | Any subdomain, single segment |

## Exclude Patterns

URLs matching any exclude pattern are removed from scope, even if they match an include pattern.

```toml
[scope]
include = ["https://api.example.com/*"]
exclude = [
  "https://api.example.com/admin/*",
  "https://api.example.com/payments/*",
]
```

This targets the API but blocks testing of admin and payment endpoints.

## Rate Limiting

| Setting | Default | Description |
|---------|---------|-------------|
| `max_rate` | 50 | Maximum requests per second across all adapters |
| `max_concurrent` | 10 | Maximum parallel connections |

## Resolution Logic

A URL is in scope when:

1. It matches at least one include pattern (or no include patterns are defined), **and**
2. It does not match any exclude pattern.

## Examples

Target staging only:

```toml
[scope]
include = ["https://staging.api.example.com/*"]
```

Target everything except production admin:

```toml
[scope]
include = ["https://*.example.com/*"]
exclude = ["https://prod.example.com/admin/*"]
```

No restrictions (test everything):

```toml
[scope]
# No include or exclude = allow all
```
