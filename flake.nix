{
  description = "netinject — lightweight API security testing orchestrator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # --- Security tools we want available in dev shell ---
        securityTools = with pkgs; [
          # Fuzzing
          ffuf

          # Vulnerability scanning
          httpx
          nuclei

          # SQL injection
          sqlmap

          # Network utilities
          nmap
          curl
          jq

          # MITM proxy (mitmproxy pulls in python)
          mitmproxy
        ];

        # --- Build dependencies ---
        buildDeps = with pkgs; [
          pkg-config
          openssl
        ];

        # --- Dev tools ---
        devTools = with pkgs; [
          rustToolchain
          cargo-watch
          cargo-edit
          cargo-outdated
          cargo-audit
          cargo-nextest

          # Linting
          rustfmt
          clippy

          # Nix
          nil
          nixpkgs-fmt
        ];

        # --- Common env args ---
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = buildDeps;
          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          ];

          # SQLite needs to be found
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        # --- Cargo artifacts ---
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # --- The actual binary ---
        netinject = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

      in
      {
        checks = {
          inherit netinject;

          netinject-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });

          netinject-fmt = craneLib.cargoFmt {
            src = craneLib.cleanCargoSource ./.;
          };

          netinject-test = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            src = craneLib.cleanCargoSource ./.;
            partitions = 1;
            partitionType = "count";
          });
        };

        packages = {
          default = netinject;
          netinject = netinject;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = netinject;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ netinject ];

          nativeBuildInputs = buildDeps ++ devTools;

          buildInputs = with pkgs; [
            openssl
          ] ++ securityTools;

          env = {
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          };

          shellHook = ''
            echo "╔══════════════════════════════════════╗"
            echo "║         netinject dev shell          ║"
            echo "╚══════════════════════════════════════╝"
            echo ""

            # Check tool availability
            echo "Security tools:"
            ${pkgs.lib.concatMapStrings (tool: ''
              if command -v ${tool} &>/dev/null; then
                echo "  ✓ ${tool}  $("${tool}" --version 2>&1 | head -1)"
              else
                echo "  ✗ ${tool}  not found"
              fi
            '') [ "ffuf" "sqlmap" "mitmdump" "nmap" "curl" "jq" "httpx" "nuclei" ]}

            echo ""
            echo "Rust: $(rustc --version) ($(rustup show active-toolchain 2>/dev/null || echo 'system'))"
            echo ""
            echo "Run 'cargo watch -x check' for live type checking"
            echo "Run 'cargo watch -x test' for live testing"
          '';
        };
      }
    );
}
