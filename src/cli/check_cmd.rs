use anyhow::Result;

use crate::adapters::AdapterRegistry;
use crate::cli::args::Cli;

/// Tool definitions for the check command: (binary_name, display_name, version_flag).
const TOOLS: &[(&str, &str, &str)] = &[
    ("ffuf", "ffuf", "--version"),
    ("nuclei", "nuclei", "-version"),
    ("httpx", "httpx", "-version"),
    ("sqlmap", "sqlmap", "--version"),
    ("mitmdump", "mitmproxy", "--version"),
];

pub fn run(_cli: &Cli) -> Result<()> {
    let registry = AdapterRegistry::new();
    let mut all_found = true;

    println!("Checking tool installations...\n");

    for (binary, display_name, _version_flag) in TOOLS {
        match registry.get(display_name) {
            Some(adapter) => match adapter.check_available() {
                Ok(true) => {
                    let version = adapter.version().unwrap_or_else(|_| "unknown".into());
                    let path = which::which(binary)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| "unknown".into());
                    println!("  ✓ {display_name:<12} {version:<20} {path}");
                }
                Ok(false) => {
                    println!("  ✗ {display_name:<12} not found");
                    all_found = false;
                }
                Err(e) => {
                    println!("  ✗ {display_name:<12} error: {e}");
                    all_found = false;
                }
            },
            None => {
                println!("  ✗ {display_name:<12} adapter not registered");
                all_found = false;
            }
        }
    }

    println!();
    if all_found {
        println!("All tools are installed and available.");
    } else {
        println!("Some tools are missing. Install them to enable full functionality.");
    }

    Ok(())
}
