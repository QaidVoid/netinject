use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::Finding;

pub struct HttpxAdapter;

#[async_trait]
impl Adapter for HttpxAdapter {
    fn name(&self) -> &str {
        "httpx"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(which::which("httpx").is_ok())
    }

    fn version(&self) -> Result<String> {
        let output = std::process::Command::new("httpx")
            .arg("-version")
            .output()
            .context("failed to run httpx -version")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.httpx;
        let findings = Vec::new();

        let mut cmd = std::process::Command::new("httpx");
        cmd.arg("-json")
            .arg("-silent")
            .arg("-threads")
            .arg(config.threads.to_string())
            .arg("-rate-limit")
            .arg(config.rate_limit.to_string());

        if config.tech_detect {
            cmd.arg("-tech-detect");
        }

        for url in &urls {
            cmd.arg(url);
        }

        // TODO: parse httpx JSONL output into findings
        let _ = cmd;
        let _ = ctx;

        tracing::info!(adapter = "httpx", count = urls.len(), "would run httpx");

        Ok(findings)
    }
}
