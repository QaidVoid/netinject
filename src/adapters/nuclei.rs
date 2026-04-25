use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::Finding;

pub struct NucleiAdapter;

#[async_trait]
impl Adapter for NucleiAdapter {
    fn name(&self) -> &str {
        "nuclei"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(which::which("nuclei").is_ok())
    }

    fn version(&self) -> Result<String> {
        let output = std::process::Command::new("nuclei")
            .arg("-version")
            .output()
            .context("failed to run nuclei -version")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.nuclei;
        let findings = Vec::new();

        for url in &urls {
            let mut cmd = std::process::Command::new("nuclei");
            cmd.arg("-u")
                .arg(url)
                .arg("-jsonl")
                .arg("-silent")
                .arg("-rate-limit")
                .arg(config.rate_limit.to_string());

            if !config.templates.is_empty() {
                cmd.arg("-t").args(&config.templates);
            }
            if !config.severity.is_empty() {
                cmd.arg("-severity").args(&config.severity);
            }

            // TODO: parse nuclei JSONL output into findings
            let _ = cmd;
            let _ = ctx;

            tracing::info!(adapter = "nuclei", url = %url, "would run nuclei");
        }

        Ok(findings)
    }
}
