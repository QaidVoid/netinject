use anyhow::Result;

use crate::cli::args::{Cli, SessionCommands};
use crate::cli::helpers;
use crate::report;

pub async fn run(cli: &Cli, subcommand: &SessionCommands) -> Result<()> {
    match subcommand {
        SessionCommands::List => {
            let home_dir = helpers::ensure_home_dir()?;
            let store = helpers::open_session_store(&home_dir)?;
            let sessions = store.list_sessions()?;
            if sessions.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }

            let rows: Vec<(String, String, String, String, String)> = sessions
                .iter()
                .map(|s| {
                    let short_id = &s.id.to_string()[..8];
                    let time = s.created_at.format("%Y-%m-%d %H:%M").to_string();
                    let pipeline = s.pipeline.clone().unwrap_or_else(|| "-".into());
                    let duration = s
                        .duration_ms
                        .map(|d| format!("{:.1}s", d as f64 / 1000.0))
                        .unwrap_or_else(|| "-".into());
                    (short_id.into(), time, s.target.clone(), pipeline, duration)
                })
                .collect();

            let mut table = tabled::Table::new(rows);
            table.with(tabled::settings::Style::modern());
            println!("{table}");
        }
        SessionCommands::Show { id } => {
            let home_dir = helpers::ensure_home_dir()?;
            let store = helpers::open_session_store(&home_dir)?;

            let session_id = helpers::resolve_session_id(&store, id)
                .ok_or_else(|| anyhow::anyhow!("session '{id}' not found"))?;
            let session = store.get_session(session_id)?;
            let findings = store.get_findings(session_id)?;

            // Print session info
            println!("Session: {}", session.id);
            println!("Target:  {}", session.target);
            println!("Status:  {}", session.status);
            if let Some(ref p) = session.pipeline {
                println!("Pipeline: {p}");
            }
            if let Some(d) = session.duration_ms {
                println!("Duration: {:.1}s", d as f64 / 1000.0);
            }
            println!(
                "Created: {}",
                session.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!();

            if findings.is_empty() {
                println!("No findings recorded.");
                return Ok(());
            }

            // Output findings using the report system
            let output = report::write_findings(&findings, cli.format)?;
            print!("{output}");
        }
        SessionCommands::Diff { id_a, id_b } => {
            let home_dir = helpers::ensure_home_dir()?;
            let store = helpers::open_session_store(&home_dir)?;

            let sid_a = helpers::resolve_session_id(&store, id_a)
                .ok_or_else(|| anyhow::anyhow!("session '{id_a}' not found"))?;
            let sid_b = helpers::resolve_session_id(&store, id_b)
                .ok_or_else(|| anyhow::anyhow!("session '{id_b}' not found"))?;

            let findings_a = store.get_findings(sid_a)?;
            let findings_b = store.get_findings(sid_b)?;

            println!("Comparing sessions:");
            println!("  A: {} ({} findings)", id_a, findings_a.len());
            println!("  B: {} ({} findings)", id_b, findings_b.len());
            println!();

            // Build URL sets for comparison
            let urls_a: std::collections::HashSet<String> =
                findings_a.iter().map(|f| f.url.clone()).collect();
            let urls_b: std::collections::HashSet<String> =
                findings_b.iter().map(|f| f.url.clone()).collect();

            // New in B (not in A)
            let new_urls: Vec<_> = urls_b.difference(&urls_a).collect();
            // Removed from A (not in B)
            let removed_urls: Vec<_> = urls_a.difference(&urls_b).collect();
            // Common
            let common_count = urls_a.intersection(&urls_b).count();

            if !new_urls.is_empty() {
                println!("New findings in B:");
                for url in &new_urls {
                    let finding = findings_b.iter().find(|f| &f.url == *url).unwrap();
                    println!("  + [{}] {} {}", finding.severity, finding.source, url);
                }
            }

            if !removed_urls.is_empty() {
                println!("Findings removed in B:");
                for url in &removed_urls {
                    let finding = findings_a.iter().find(|f| &f.url == *url).unwrap();
                    println!("  - [{}] {} {}", finding.severity, finding.source, url);
                }
            }

            if new_urls.is_empty() && removed_urls.is_empty() {
                println!("No differences between sessions ({common_count} common findings).");
            } else {
                println!(
                    "\nSummary: +{} new, -{} removed, {common_count} unchanged",
                    new_urls.len(),
                    removed_urls.len()
                );
            }
        }
        SessionCommands::Export { id } => {
            let home_dir = helpers::ensure_home_dir()?;
            let store = helpers::open_session_store(&home_dir)?;

            let session_id = helpers::resolve_session_id(&store, id)
                .ok_or_else(|| anyhow::anyhow!("session '{id}' not found"))?;
            let findings = store.get_findings(session_id)?;

            if findings.is_empty() {
                println!("No findings to export.");
                return Ok(());
            }

            let output = report::write_findings(&findings, cli.format)?;

            if let Some(ref path) = cli.output {
                std::fs::write(path, &output)?;
                println!("Exported {} findings to {path}", findings.len());
            } else {
                // Default: write to a file named by session id
                let filename = format!("{id}-findings.jsonl");
                std::fs::write(&filename, &output)?;
                println!("Exported {} findings to {filename}", findings.len());
            }
        }
    }
    Ok(())
}
