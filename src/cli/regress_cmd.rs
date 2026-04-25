use anyhow::Result;

use crate::baseline::{Change, DiffSeverity, EndpointDiff, RegressionReport};
use crate::cli::args::Cli;
use crate::cli::helpers;
use crate::config;
use crate::finding::{Category, Finding, Severity};

pub async fn run(cli: &Cli, against: Option<&str>, paths: Option<&str>) -> Result<()> {
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

    // Load baseline (most recent, or specified by --against)
    let baseline = if let Some(id) = against {
        resolve_baseline(&store, id).ok_or_else(|| anyhow::anyhow!("baseline '{id}' not found"))?
    } else {
        let baselines = store.list_baselines()?;
        baselines.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!("no baselines captured. Run `netinject baseline capture` first")
        })?
    };

    println!(
        "Checking against baseline {} ({} entries)...",
        &baseline.id.to_string()[..8],
        baseline.entries.len()
    );

    // Filter endpoints by --paths if specified
    let entries_to_check: Vec<_> = if let Some(filter) = paths {
        let filter_paths: Vec<&str> = filter.split(',').map(|s| s.trim()).collect();
        baseline
            .entries
            .iter()
            .filter(|e| filter_paths.iter().any(|p| e.path.contains(p)))
            .collect()
    } else {
        baseline.entries.iter().collect()
    };

    if entries_to_check.is_empty() {
        println!("No matching endpoints to check.");
        return Ok(());
    }

    // Probe current state of each endpoint
    let auth_headers = helpers::resolve_auth_headers(&cfg, cli.auth.as_deref());
    let mut diffs = Vec::new();

    for entry in &entries_to_check {
        let current = match super::baseline_cmd::probe_endpoint_internal(
            &entry.method,
            &entry.path,
            &auth_headers,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(path = %entry.path, error = %e, "failed to probe endpoint");
                let changes = vec![Change::BodyChanged {
                    from_hash: entry.body_hash.clone(),
                    to_hash: "error".into(),
                }];
                diffs.push(EndpointDiff {
                    method: entry.method.clone(),
                    path: entry.path.clone(),
                    changes,
                    severity: DiffSeverity::Warning,
                });
                continue;
            }
        };

        let changes = compare_entries(entry, &current);
        if !changes.is_empty() {
            let severity = determine_severity(&changes, &cfg.regression);
            diffs.push(EndpointDiff {
                method: entry.method.clone(),
                path: entry.path.clone(),
                changes,
                severity,
            });
        }
    }

    let report = RegressionReport {
        baseline_id: baseline.id,
        checked_at: chrono::Utc::now(),
        target: target.to_string(),
        diffs: diffs.clone(),
    };

    if diffs.is_empty() {
        println!(
            "No regressions detected. All {} endpoints match baseline.",
            entries_to_check.len()
        );
        return Ok(());
    }

    // Display results
    println!("\nRegressions detected:\n");
    for diff in &diffs {
        let severity_str = match diff.severity {
            DiffSeverity::Breaking => "BREAKING",
            DiffSeverity::Warning => "WARNING",
            DiffSeverity::Info => "INFO",
        };
        println!("  [{severity_str}] {} {}", diff.method, diff.path);
        for change in &diff.changes {
            match change {
                Change::StatusCodeChanged { from, to } => {
                    println!("       status: {from} -> {to}");
                }
                Change::HeaderAdded(h) => {
                    println!("       header added: {h}");
                }
                Change::HeaderRemoved(h) => {
                    println!("       header removed: {h}");
                }
                Change::SchemaDrift { description } => {
                    println!("       schema drift: {description}");
                }
                Change::TimingAnomaly {
                    baseline_ms,
                    current_ms,
                } => {
                    println!("       timing: {baseline_ms}ms -> {current_ms}ms");
                }
                Change::BodyChanged { .. } => {
                    println!("       body content changed");
                }
            }
        }
    }

    let breaking = diffs
        .iter()
        .filter(|d| d.severity == DiffSeverity::Breaking)
        .count();
    let warnings = diffs
        .iter()
        .filter(|d| d.severity == DiffSeverity::Warning)
        .count();
    let info = diffs
        .iter()
        .filter(|d| d.severity == DiffSeverity::Info)
        .count();
    println!(
        "\nSummary: {breaking} breaking, {warnings} warnings, {info} info out of {} endpoints",
        entries_to_check.len()
    );

    // Store findings from the regression
    let (session_id, session) =
        helpers::create_session(&store, target, "regress", &serde_json::to_string(&cfg)?)?;

    let mut findings = Vec::new();
    for diff in &report.diffs {
        let mut finding = Finding::new(session_id, "regress");
        finding.category = Category::Regression;
        finding.url = diff.path.clone();
        finding.method = Some(diff.method.clone());
        finding.title = format!(
            "{} {} - {} change(s)",
            diff.method,
            diff.path,
            diff.changes.len()
        );
        finding.description = diff
            .changes
            .iter()
            .map(|c| match c {
                Change::StatusCodeChanged { from, to } => format!("status: {from} -> {to}"),
                Change::HeaderAdded(h) => format!("header added: {h}"),
                Change::HeaderRemoved(h) => format!("header removed: {h}"),
                Change::SchemaDrift { description } => format!("schema: {description}"),
                Change::TimingAnomaly {
                    baseline_ms,
                    current_ms,
                } => format!("timing: {baseline_ms}ms -> {current_ms}ms"),
                Change::BodyChanged { .. } => "body changed".into(),
            })
            .collect::<Vec<_>>()
            .join("; ");
        finding.severity = match diff.severity {
            DiffSeverity::Breaking => Severity::High,
            DiffSeverity::Warning => Severity::Medium,
            DiffSeverity::Info => Severity::Low,
        };
        finding.evidence = Some(serde_json::to_string(&diff)?);
        findings.push(finding);
    }

    for f in &findings {
        store.insert_finding(f)?;
    }
    helpers::complete_session(&store, &session)?;

    helpers::output_findings(&findings, cli)?;
    Ok(())
}

/// Compare a baseline entry with the current response.
fn compare_entries(
    baseline: &crate::baseline::BaselineEntry,
    current: &crate::baseline::BaselineEntry,
) -> Vec<Change> {
    let mut changes = Vec::new();

    // Status code change
    if baseline.status_code != current.status_code {
        changes.push(Change::StatusCodeChanged {
            from: baseline.status_code,
            to: current.status_code,
        });
    }

    // Body content change
    if baseline.body_hash != current.body_hash {
        changes.push(Change::BodyChanged {
            from_hash: baseline.body_hash.clone(),
            to_hash: current.body_hash.clone(),
        });
    }

    // Schema drift (if both have schema hashes)
    if let (Some(ref base_schema), Some(ref curr_schema)) = (
        baseline.body_schema_hash.as_ref(),
        current.body_schema_hash.as_ref(),
    ) && base_schema != curr_schema
    {
        changes.push(Change::SchemaDrift {
            description: "response schema structure changed".into(),
        });
    }

    // Timing anomaly (>2x slowdown)
    if baseline.response_time_ms > 0 && current.response_time_ms > baseline.response_time_ms * 2 {
        changes.push(Change::TimingAnomaly {
            baseline_ms: baseline.response_time_ms,
            current_ms: current.response_time_ms,
        });
    }

    // Header diffs
    let base_headers: std::collections::HashSet<String> = baseline
        .headers
        .iter()
        .map(|(k, _)| k.to_lowercase())
        .collect();
    let curr_headers: std::collections::HashSet<String> = current
        .headers
        .iter()
        .map(|(k, _)| k.to_lowercase())
        .collect();

    for h in curr_headers.difference(&base_headers) {
        changes.push(Change::HeaderAdded(h.clone()));
    }
    for h in base_headers.difference(&curr_headers) {
        changes.push(Change::HeaderRemoved(h.clone()));
    }

    changes
}

/// Determine the severity of a set of changes based on regression config.
fn determine_severity(
    changes: &[Change],
    config: &crate::config::RegressionConfig,
) -> DiffSeverity {
    for change in changes {
        let severity = match change {
            Change::StatusCodeChanged { .. } => parse_diff_severity(&config.status_code_change),
            Change::SchemaDrift { .. } => parse_diff_severity(&config.schema_drift),
            Change::TimingAnomaly { .. } => DiffSeverity::Info,
            Change::HeaderAdded(_) | Change::HeaderRemoved(_) => {
                parse_diff_severity(&config.header_change)
            }
            Change::BodyChanged { .. } => parse_diff_severity(&config.body_hash_change),
        };
        // Return the highest severity found
        if severity == DiffSeverity::Breaking {
            return DiffSeverity::Breaking;
        }
    }

    // Check if any are warnings
    for change in changes {
        let severity = match change {
            Change::StatusCodeChanged { .. } => parse_diff_severity(&config.status_code_change),
            Change::BodyChanged { .. } => parse_diff_severity(&config.body_hash_change),
            _ => DiffSeverity::Info,
        };
        if severity == DiffSeverity::Warning {
            return DiffSeverity::Warning;
        }
    }

    DiffSeverity::Info
}

fn parse_diff_severity(s: &str) -> DiffSeverity {
    match s.to_lowercase().as_str() {
        "breaking" => DiffSeverity::Breaking,
        "warning" => DiffSeverity::Warning,
        _ => DiffSeverity::Info,
    }
}

/// Resolve a baseline ID (full or short prefix).
fn resolve_baseline(
    store: &crate::session::store::SessionStore,
    id: &str,
) -> Option<crate::baseline::BaselineSnapshot> {
    if let Ok(uuid) = uuid::Uuid::parse_str(id)
        && let Ok(b) = store.get_baseline(uuid)
    {
        return Some(b);
    }

    let baselines = store.list_baselines().ok()?;
    let matches: Vec<_> = baselines
        .into_iter()
        .filter(|b| b.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        1 => Some(matches.into_iter().next().unwrap()),
        _ => None,
    }
}
