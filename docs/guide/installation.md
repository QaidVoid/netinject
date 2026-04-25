# Installation

## From Source (Recommended)

Clone and build with Cargo:

```bash
git clone https://github.com/QaidVoid/netinject.git
cd netinject
cargo build --release
```

The binary will be at `target/release/netinject`.

## With Nix

If you use Nix, a flake is provided:

```bash
# Build
nix build

# Run directly
nix run

# Dev shell (includes security tools)
nix develop
```

## Verify Installation

```bash
netinject --version
```

## Check Tool Availability

Run the check command to verify which tools are installed:

```bash
netinject check
```

Output:

```
  ✓ ffuf    v2.1.0   /usr/local/bin/ffuf
  ✓ nuclei  v3.8.0   /usr/local/bin/nuclei
  ✓ httpx   v1.9.0   /usr/local/bin/httpx
  ✗ sqlmap  not found, install with: pip install sqlmap
  ✗ mitmdump not found, install with: pip install mitmproxy
```
