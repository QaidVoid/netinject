use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::Finding;

pub struct MitmproxyAdapter;

#[async_trait]
impl Adapter for MitmproxyAdapter {
    fn name(&self) -> &str {
        "mitmproxy"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(which::which("mitmdump").is_ok())
    }

    fn version(&self) -> Result<String> {
        let output = std::process::Command::new("mitmdump")
            .arg("--version")
            .output()
            .context("failed to run mitmdump --version")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, _input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let config = &ctx.config.adapters.mitmproxy;
        tracing::info!(
            adapter = "mitmproxy",
            host = %config.listen_host,
            port = %config.listen_port,
            "would start mitmdump"
        );
        Ok(vec![])
    }
}
