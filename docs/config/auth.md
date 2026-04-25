# Auth Profiles

Auth profiles let you define authentication methods that netinject injects as HTTP headers when running adapters.

## Profile Types

### Bearer Token

```toml
[[auth]]
name = "staging"
type = "bearer"
token = "eyJhbGciOiJIUzI1NiIs..."
```

Sends `Authorization: Bearer <token>` with every request.

### Basic Auth

```toml
[[auth]]
name = "dev"
type = "basic"
username = "admin"
password = "s3cret"
```

Sends `Authorization: Basic <base64(user:pass)>` with every request.

### API Key

```toml
[[auth]]
name = "api-prod"
type = "api-key"
header = "X-API-Key"
key = "abc123def456"
```

Sends a custom header (`X-API-Key: abc123def456`) with every request.

### OAuth2

```toml
[[auth]]
name = "oauth"
type = "oauth2"
token = "${OAUTH_ACCESS_TOKEN}"
```

OAuth2 support is reserved for future implementation. Currently behaves like bearer.

## Environment Variables

Use `${VAR_NAME}` syntax to reference environment variables. The value is resolved at runtime:

```toml
[[auth]]
name = "staging"
type = "bearer"
token = "${STAGING_API_TOKEN}"
```

If the variable is not set, the literal `${STAGING_API_TOKEN}` string is used as-is (which will likely cause auth failures).

## Using Auth Profiles

Pass the profile name via the `--auth` flag:

```bash
netinject scan --target https://api.example.com --auth staging
netinject run --target https://api.example.com --auth dev
```

## Multiple Profiles

You can define multiple profiles in the same config file. Only one can be active per run:

```toml
[[auth]]
name = "dev"
type = "bearer"
token = "${DEV_TOKEN}"

[[auth]]
name = "staging"
type = "bearer"
token = "${STAGING_TOKEN}"

[[auth]]
name = "prod"
type = "api-key"
header = "X-API-Key"
key = "${PROD_API_KEY}"
```
