use anyhow::Result;
use std::io::Write;

use crate::cli::args::Cli;

const CONFIG_TEMPLATE: &str = r#"# netinject.toml — project-level config

[project]
name = "my-api"
target = "https://api.staging.example.com"
spec = "./openapi.yaml"

[scope]
include = ["https://api.staging.example.com/*"]
exclude = ["https://api.staging.example.com/admin/*"]
max_rate = 50
max_concurrent = 10

# --- Auth profiles ---

# [[auth]]
# name = "staging"
# type = "bearer"
# token = "${STAGING_TOKEN}"

# --- Adapter configs ---

[adapters.ffuf]
wordlist = "/usr/share/seclists/Discovery/Web-Content/common.txt"
threads = 40
timeout = 10
recursive = false

[adapters.nuclei]
templates = ["cves/", "vulnerabilities/", "misconfiguration/"]
severity = ["high", "critical"]
rate_limit = 100

[adapters.httpx]
threads = 50
rate_limit = 150
tech_detect = true

[adapters.sqlmap]
level = 3
risk = 2
batch = true

[adapters.mitmproxy]
listen_host = "127.0.0.1"
listen_port = 8080
upstream_proxy = ""

# --- Pipelines ---

[[pipeline]]
name = "full-api-scan"
description = "Full API security scan"
steps = [
  { adapter = "httpx", label = "recon" },
  { adapter = "ffuf", label = "fuzz", depends_on = "recon" },
  { adapter = "nuclei", label = "scan", depends_on = "recon" },
]

[[pipeline]]
name = "quick-scan"
description = "Fast vulnerability scan only"
steps = [
  { adapter = "nuclei" },
]

# --- Regression ---

[regression]
status_code_change = "breaking"
schema_drift = "breaking"
timing_threshold = 2.0
header_change = "info"
body_hash_change = "warning"
"#;

pub fn run(_cli: &Cli, name: Option<&str>) -> Result<()> {
    let project_name = name.unwrap_or("my-api");
    let config_content = CONFIG_TEMPLATE.replace("my-api", project_name);

    // Write netinject.toml
    let config_path = std::path::Path::new("netinject.toml");
    if config_path.exists() {
        anyhow::bail!("netinject.toml already exists in the current directory");
    }

    let mut file = std::fs::File::create(config_path)?;
    file.write_all(config_content.as_bytes())?;

    // Create directories
    std::fs::create_dir_all(".netinject")?;

    println!("✓ Created netinject.toml");
    println!("✓ Created .netinject/ directory");
    println!();
    println!("Next steps:");
    println!("  1. Edit netinject.toml with your target and scope");
    println!("  2. Add your OpenAPI spec file and update the 'spec' path");
    println!("  3. Run 'netinject check' to verify tool installations");
    println!("  4. Run 'netinject run' to start testing");
    println!();
    println!("Config is auto-discovered from the current directory.");

    Ok(())
}
