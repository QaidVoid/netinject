use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::Finding;

pub struct SqlmapAdapter;

#[async_trait]
impl Adapter for SqlmapAdapter {
    fn name(&self) -> &str {
        "sqlmap"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(which::which("sqlmap").is_ok())
    }

    fn version(&self) -> Result<String> {
        let output = std::process::Command::new("sqlmap")
            .arg("--version")
            .output()
            .context("failed to run sqlmap --version")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.sqlmap;

        for url in &urls {
            tracing::info!(adapter = "sqlmap", url = %url, "would run sqlmap");
            let _ = config;
            let _ = ctx;
        }

        Ok(vec![])
    }
}
