use anyhow::Result;

use crate::adapters::{AdapterInput, AdapterRegistry, RunContext};
use crate::cli::args::Cli;
use crate::cli::helpers;
use crate::config;

pub async fn run(cli: &Cli) -> Result<()> {
    let cfg = config::resolve_config(
        cli.target.as_deref(),
        cli.spec.as_deref(),
        cli.config.as_deref().map(std::path::Path::new),
    )?;

    let target = cfg.project.target.as_deref().ok_or_else(|| {
        anyhow::anyhow!("no target URL specified. Use --target or set it in config")
    })?;

    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;
    let (session_id, session) =
        helpers::create_session(&store, target, "scan", &serde_json::to_string(&cfg)?)?;

    let auth_headers = helpers::resolve_auth_headers(&cfg, cli.auth.as_deref());

    let registry = AdapterRegistry::new();
    let adapter = registry
        .get("nuclei")
        .ok_or_else(|| anyhow::anyhow!("nuclei adapter not registered"))?;

    if !adapter.check_available()? {
        anyhow::bail!(
            "nuclei is not installed. Install it: go install -v github.com/projectdiscovery/nuclei/v3/cmd/nuclei@latest"
        );
    }

    if cli.dry_run {
        println!("Would run: nuclei -u {target} -jsonl -silent");
        helpers::complete_session(&store, &session)?;
        return Ok(());
    }

    let ctx = RunContext {
        session_id,
        config: cfg.clone(),
        auth_headers,
        extra_args: std::collections::HashMap::new(),
    };

    let findings = adapter
        .run(AdapterInput::Urls(vec![target.to_string()]), &ctx)
        .await?;

    for finding in &findings {
        store.insert_finding(finding)?;
    }

    helpers::complete_session(&store, &session)?;
    helpers::output_findings(&findings, cli)?;

    Ok(())
}
