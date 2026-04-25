use anyhow::Result;

use crate::baseline::{BaselineEntry, BaselineSnapshot};
use crate::cli::args::{BaselineCommands, Cli};
use crate::cli::helpers;
use crate::config;

pub async fn run(cli: &Cli, subcommand: &BaselineCommands) -> Result<()> {
    match subcommand {
        BaselineCommands::Capture => capture(cli).await,
        BaselineCommands::List => list(cli),
        BaselineCommands::Diff {
            baseline_a,
            baseline_b,
        } => diff(cli, baseline_a, baseline_b),
    }
}

async fn capture(cli: &Cli) -> Result<()> {
    let cfg = config::resolve_config(
        cli.target.as_deref(),
        cli.spec.as_deref(),
        cli.config.as_deref().map(std::path::Path::new),
    )?;

    let target = cfg.project.target.as_deref().ok_or_else(|| {
        anyhow::anyhow!("no target URL specified. Use --target or set it in config")
    })?;

    // Determine endpoints to probe
    let endpoints = if let Some(ref spec_path) = cfg.project.spec {
        let spec = crate::spec::parse_openapi(std::path::Path::new(spec_path))?;
        let base = spec.base_url.as_deref().unwrap_or(target);
        spec.endpoints
            .iter()
            .map(|e| {
                let url = format!("{base}{}", e.path);
                (e.method.to_string(), url)
            })
            .collect::<Vec<_>>()
    } else {
        // Without a spec, just probe the root and common paths
        vec![
            ("GET".into(), target.trim_end_matches('/').to_string()),
            (
                "GET".into(),
                format!("{}/health", target.trim_end_matches('/')),
            ),
            (
                "GET".into(),
                format!("{}/api", target.trim_end_matches('/')),
            ),
            (
                "GET".into(),
                format!("{}/docs", target.trim_end_matches('/')),
            ),
        ]
    };

    if cli.dry_run {
        println!("Would capture baseline for {} endpoints:", endpoints.len());
        for (method, url) in &endpoints {
            println!("  {method} {url}");
        }
        return Ok(());
    }

    // Resolve auth headers
    let auth_headers = helpers::resolve_auth_headers(&cfg, cli.auth.as_deref());

    println!("Capturing baseline for {} endpoints...", endpoints.len());

    let mut entries = Vec::new();
    for (method, url) in &endpoints {
        match probe_endpoint_internal(method, url, &auth_headers) {
            Ok(entry) => {
                println!(
                    "  {} {} [{}] {}ms",
                    method, entry.path, entry.status_code, entry.response_time_ms
                );
                entries.push(entry);
            }
            Err(e) => {
                tracing::warn!(method, url, error = %e, "failed to probe endpoint");
                println!("  {method} {url} [ERROR] {e}");
            }
        }
    }

    // Compute spec hash (hash of the spec content, or empty if no spec)
    let spec_hash = cfg
        .project
        .spec
        .as_deref()
        .map(|path| {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            helpers::sha256_hex(&content)
        })
        .unwrap_or_default();

    let baseline = BaselineSnapshot {
        id: uuid::Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        target: target.to_string(),
        spec_hash,
        entries,
    };

    // Store baseline
    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;
    store.insert_baseline(&baseline)?;

    println!(
        "\nBaseline captured: {} ({} entries)",
        &baseline.id.to_string()[..8],
        baseline.entries.len()
    );
    Ok(())
}

fn list(_cli: &Cli) -> Result<()> {
    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;

    let baselines = store.list_baselines()?;
    if baselines.is_empty() {
        println!("No baselines captured.");
        return Ok(());
    }

    let rows: Vec<(String, String, String, String)> = baselines
        .iter()
        .map(|b| {
            let short_id = &b.id.to_string()[..8];
            let time = b.created_at.format("%Y-%m-%d %H:%M").to_string();
            let n = b.entries.len().to_string();
            (short_id.into(), time, b.target.clone(), n)
        })
        .collect();

    let mut table = tabled::Table::new(rows);
    table.with(tabled::settings::Style::modern());
    println!("{table}");
    Ok(())
}

fn diff(_cli: &Cli, id_a: &str, id_b: &str) -> Result<()> {
    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;

    let baseline_a = helpers::resolve_baseline(&store, id_a)
        .ok_or_else(|| anyhow::anyhow!("baseline '{id_a}' not found"))?;
    let baseline_b = helpers::resolve_baseline(&store, id_b)
        .ok_or_else(|| anyhow::anyhow!("baseline '{id_b}' not found"))?;

    println!(
        "Comparing baselines:\n  A: {} ({} entries)\n  B: {} ({} entries)\n",
        &baseline_a.id.to_string()[..8],
        baseline_a.entries.len(),
        &baseline_b.id.to_string()[..8],
        baseline_b.entries.len()
    );

    // Index entries by (method, path)
    use std::collections::HashMap;
    let map_a: HashMap<(String, String), &BaselineEntry> = baseline_a
        .entries
        .iter()
        .map(|e| ((e.method.clone(), e.path.clone()), e))
        .collect();
    let map_b: HashMap<(String, String), &BaselineEntry> = baseline_b
        .entries
        .iter()
        .map(|e| ((e.method.clone(), e.path.clone()), e))
        .collect();

    let keys_a: std::collections::HashSet<_> = map_a.keys().collect();
    let keys_b: std::collections::HashSet<_> = map_b.keys().collect();

    let added = keys_b.difference(&keys_a);
    let removed = keys_a.difference(&keys_b);
    let common = keys_a.intersection(&keys_b);

    let mut changes_count = 0;

    for key in added {
        let entry = map_b[key];
        println!(
            "  + {} {} [{}]",
            entry.method, entry.path, entry.status_code
        );
        changes_count += 1;
    }

    for key in removed {
        let entry = map_a[key];
        println!(
            "  - {} {} [{}]",
            entry.method, entry.path, entry.status_code
        );
        changes_count += 1;
    }

    for key in common {
        let a = map_a[key];
        let b = map_b[key];
        let mut diffs = Vec::new();

        if a.status_code != b.status_code {
            diffs.push(format!("status: {} -> {}", a.status_code, b.status_code));
        }
        if a.body_hash != b.body_hash {
            diffs.push("body changed".into());
        }
        if a.response_time_ms > 0 && b.response_time_ms > 0 {
            let ratio = b.response_time_ms as f64 / a.response_time_ms as f64;
            if ratio > 2.0 {
                diffs.push(format!(
                    "timing: {}ms -> {}ms ({:.1}x)",
                    a.response_time_ms, b.response_time_ms, ratio
                ));
            }
        }

        // Header diffs
        let headers_a: std::collections::HashSet<_> =
            a.headers.iter().map(|(k, _)| k.to_lowercase()).collect();
        let headers_b: std::collections::HashSet<_> =
            b.headers.iter().map(|(k, _)| k.to_lowercase()).collect();
        for h in headers_b.difference(&headers_a) {
            diffs.push(format!("header added: {h}"));
        }
        for h in headers_a.difference(&headers_b) {
            diffs.push(format!("header removed: {h}"));
        }

        if !diffs.is_empty() {
            println!("  ~ {} {}", a.method, a.path);
            for d in &diffs {
                println!("      {d}");
            }
            changes_count += 1;
        }
    }

    if changes_count == 0 {
        println!("No differences between baselines.");
    } else {
        println!("\n{changes_count} endpoint(s) changed.");
    }

    Ok(())
}

/// Probe a single endpoint and capture response metadata.
/// Public so regress_cmd can reuse it.
pub fn probe_endpoint_internal(
    method: &str,
    url: &str,
    auth_headers: &[(String, String)],
) -> Result<BaselineEntry> {
    use std::io::Read;

    let start = std::time::Instant::now();

    // Build a config that does not treat 4xx/5xx as errors so we can capture them.
    let config = ureq::config::Config::builder()
        .http_status_as_error(false)
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();

    let agent = ureq::Agent::new_with_config(config);

    // For body-capable methods (POST/PUT/PATCH), send empty body.
    // For non-body methods (GET/HEAD/DELETE), just call.
    let resp = match method {
        "POST" => {
            let mut r = agent.post(url);
            for (k, v) in auth_headers {
                r = r.header(k, v);
            }
            r.send_empty()
        }
        "PUT" => {
            let mut r = agent.put(url);
            for (k, v) in auth_headers {
                r = r.header(k, v);
            }
            r.send_empty()
        }
        "PATCH" => {
            let mut r = agent.patch(url);
            for (k, v) in auth_headers {
                r = r.header(k, v);
            }
            r.send_empty()
        }
        "DELETE" => {
            let mut r = agent.delete(url);
            for (k, v) in auth_headers {
                r = r.header(k, v);
            }
            r.call()
        }
        "HEAD" => {
            let mut r = agent.head(url);
            for (k, v) in auth_headers {
                r = r.header(k, v);
            }
            r.call()
        }
        _ => {
            let mut r = agent.get(url);
            for (k, v) in auth_headers {
                r = r.header(k, v);
            }
            r.call()
        }
    };

    let resp = resp.map_err(|e| anyhow::anyhow!("request failed: {e}"))?;

    let response_time_ms = start.elapsed().as_millis() as u64;
    let status_code = resp.status();

    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
        .collect();

    // Read body and hash it
    let mut body = String::new();
    let _ = resp.into_body().into_reader().read_to_string(&mut body);
    let body_hash = helpers::sha256_hex(&body);

    // Simple schema hash: hash of the body structure (strip values from JSON)
    let body_schema_hash = helpers::schema_hash(&body);

    Ok(BaselineEntry {
        method: method.to_string(),
        path: url.to_string(),
        status_code: status_code.as_u16(),
        headers,
        body_hash,
        body_schema_hash: Some(body_schema_hash),
        response_time_ms,
    })
}
