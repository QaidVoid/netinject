# Getting Started

netinject is a lightweight CLI tool that orchestrates existing API security testing tools into unified pipelines. It wraps existing tools and adds value on top: session management, regression tracking, and unified output.

## What netinject does

- **Orchestrates** tools like ffuf, nuclei, httpx, sqlmap, and mitmproxy
- **Parses** OpenAPI/Swagger specs for intelligent, targeted testing
- **Captures** baselines and detects regressions across test runs
- **Normalizes** output from every tool into a unified `Finding` format
- **Stores** everything in a local SQLite session database

## What netinject does NOT do

- It is not another fuzzer, scanner, or proxy
- It is not a Burp Suite rewrite
- It is not a SaaS platform

## Prerequisites

netinject orchestrates **external tools**. You need at least one of these installed:

| Tool | Purpose | Install |
|------|---------|---------|
| **httpx** | HTTP probing and recon | `go install -v github.com/projectdiscovery/httpx/cmd/httpx@latest` |
| **nuclei** | Vulnerability scanning | `go install -v github.com/projectdiscovery/nuclei/v3/cmd/nuclei@latest` |
| **ffuf** | Fuzzing | `go install -v github.com/ffuf/ffuf/v2@latest` |
| **sqlmap** | SQL injection | `pip install sqlmap` |
| **mitmproxy** | MITM traffic capture | `pip install mitmproxy` |

## Next Steps

- [Installation](/guide/installation) to build or install netinject
- [Quick Start](/guide/quick-start) to go from zero to first scan in 5 minutes
