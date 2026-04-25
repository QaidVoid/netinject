use anyhow::Result;

use crate::adapters::{AdapterInput, AdapterRegistry, RunContext};
use crate::cli::args::Cli;
use crate::cli::helpers;
use crate::config;

pub async fn run(cli: &Cli, fuzz_point: Option<&str>) -> Result<()> {
    let cfg = config::resolve_config(
        cli.target.as_deref(),
        cli.spec.as_deref(),
        cli.config.as_deref().map(std::path::Path::new),
    )?;

    let target = cfg.project.target.as_deref().ok_or_else(|| {
        anyhow::anyhow!("no target URL specified. Use --target or set it in config")
    })?;

    // Build the ffuf URL with FUZZ keyword placeholder.
    // If fuzz_point is provided, inject it into the URL; otherwise append /FUZZ.
    let url = match fuzz_point {
        Some(point) => {
            // fuzz_point could be a path segment, query param, etc.
            // e.g., "path" -> https://target/FUZZ
            // e.g., "param" -> https://target/?FUZZ=test
            if point == "path" || point.is_empty() {
                format!("{target}/FUZZ")
            } else {
                format!("{target}/{point}")
            }
        }
        None => format!("{target}/FUZZ"),
    };

    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;
    let (session_id, session) =
        helpers::create_session(&store, target, "fuzz", &serde_json::to_string(&cfg)?)?;

    let auth_headers = helpers::resolve_auth_headers(&cfg, cli.auth.as_deref());

    let registry = AdapterRegistry::new();
    let adapter = registry
        .get("ffuf")
        .ok_or_else(|| anyhow::anyhow!("ffuf adapter not registered"))?;

    if !adapter.check_available()? {
        anyhow::bail!(
            "ffuf is not installed. Install it: go install github.com/ffuf/ffuf/v2@latest"
        );
    }

    if cli.dry_run {
        println!(
            "Would run: ffuf -u {url} -w {} -json -s",
            cfg.adapters.ffuf.wordlist
        );
        helpers::complete_session(&store, &session)?;
        return Ok(());
    }

    let ctx = RunContext {
        session_id,
        config: cfg.clone(),
        auth_headers,
        extra_args: std::collections::HashMap::new(),
    };

    let findings = adapter.run(AdapterInput::Urls(vec![url]), &ctx).await?;

    for finding in &findings {
        store.insert_finding(finding)?;
    }

    helpers::complete_session(&store, &session)?;
    helpers::output_findings(&findings, cli)?;

    Ok(())
}
