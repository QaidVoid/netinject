use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::Finding;

pub struct FfufAdapter;

#[async_trait]
impl Adapter for FfufAdapter {
    fn name(&self) -> &str {
        "ffuf"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(which::which("ffuf").is_ok())
    }

    fn version(&self) -> Result<String> {
        let output = std::process::Command::new("ffuf")
            .arg("--version")
            .output()
            .context("failed to run ffuf --version")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.ffuf;
        let findings = Vec::new();

        for url in &urls {
            let mut cmd = std::process::Command::new("ffuf");
            cmd.arg("-u")
                .arg(url)
                .arg("-w")
                .arg(&config.wordlist)
                .arg("-t")
                .arg(config.threads.to_string())
                .arg("-timeout")
                .arg(config.timeout.to_string())
                .arg("-json");

            if config.recursive {
                cmd.arg("-recursion").arg("1");
            }

            // TODO: parse ffuf JSON output into findings
            let _ = cmd;
            let _ = ctx;

            tracing::info!(adapter = "ffuf", url = %url, "would run ffuf");
        }

        Ok(findings)
    }
}
