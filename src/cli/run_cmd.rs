use anyhow::Result;

use crate::adapters::AdapterRegistry;
use crate::cli::args::Cli;
use crate::cli::helpers;
use crate::config;

pub async fn run(cli: &Cli, pipeline: Option<&str>, target_file: Option<&str>) -> Result<()> {
    let cfg = config::resolve_config(
        cli.target.as_deref(),
        cli.spec.as_deref(),
        cli.config.as_deref().map(std::path::Path::new),
    )?;

    let pipeline_name = pipeline.unwrap_or("full-api-scan");

    // Collect initial URLs from target, target-file, or config
    let mut urls = Vec::new();

    if let Some(target) = cfg.project.target.as_deref() {
        urls.push(target.to_string());
    }

    if let Some(path) = target_file {
        let file_urls = read_url_file(path)?;
        urls.extend(file_urls);
    }

    if urls.is_empty() {
        anyhow::bail!(
            "no target URLs specified. Use --target, --target-file, or set target in config"
        );
    }

    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;
    let (session_id, session) = helpers::create_session(
        &store,
        &urls.join(", "),
        pipeline_name,
        &serde_json::to_string(&cfg)?,
    )?;

    let auth_headers = helpers::resolve_auth_headers(&cfg, cli.auth.as_deref());
    let registry = AdapterRegistry::new();

    // Check that all adapters in the pipeline are available (unless dry-run)
    if !cli.dry_run {
        let pipeline_def = cfg
            .pipeline
            .iter()
            .find(|p| p.name == pipeline_name)
            .ok_or_else(|| anyhow::anyhow!("pipeline '{pipeline_name}' not found in config"))?;

        for step in &pipeline_def.steps {
            let adapter = registry.get(&step.adapter).ok_or_else(|| {
                anyhow::anyhow!("adapter '{}' not found in registry", step.adapter)
            })?;

            if !adapter.check_available()? {
                anyhow::bail!(
                    "'{}' is not installed. Install it before running this pipeline.",
                    step.adapter
                );
            }
        }
    }

    let ctx = crate::adapters::RunContext {
        session_id,
        config: cfg.clone(),
        auth_headers,
        extra_args: std::collections::HashMap::new(),
    };

    tracing::info!(
        pipeline = %pipeline_name,
        targets = urls.len(),
        dry_run = cli.dry_run,
        "executing pipeline"
    );

    let findings =
        crate::pipeline::execute_pipeline(pipeline_name, &cfg, &registry, &ctx, urls, cli.dry_run)
            .await?;

    // Save findings
    for finding in &findings {
        store.insert_finding(finding)?;
    }

    helpers::complete_session(&store, &session)?;
    helpers::output_findings(&findings, cli)?;

    Ok(())
}

/// Read URLs from a file (one per line, ignoring blanks and comments).
fn read_url_file(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let urls = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect();
    Ok(urls)
}
