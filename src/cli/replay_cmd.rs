use anyhow::Result;

use crate::cli::args::Cli;
use crate::cli::helpers;
use crate::finding::{Category, Finding, Severity};

pub async fn run(cli: &Cli, session_id: &str, modify: Option<&Vec<String>>) -> Result<()> {
    let home_dir = helpers::ensure_home_dir()?;
    let store = helpers::open_session_store(&home_dir)?;

    // Resolve session
    let sid = helpers::resolve_session_id(&store, session_id)
        .ok_or_else(|| anyhow::anyhow!("session '{session_id}' not found"))?;
    let session = store.get_session(sid)?;
    let findings = store.get_findings(sid)?;

    if findings.is_empty() {
        println!("No findings in session to replay.");
        return Ok(());
    }

    // Parse --modify headers (format: "Header-Name: value")
    let extra_headers: Vec<(String, String)> = modify
        .map(|mods| {
            mods.iter()
                .filter_map(|m| {
                    let parts: Vec<&str> = m.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                    } else {
                        tracing::warn!(modify = %m, "invalid header format, expected 'Name: value'");
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    println!(
        "Replaying {} findings from session {}...",
        findings.len(),
        &session_id[..session_id.len().min(8)]
    );
    if !extra_headers.is_empty() {
        println!("Extra headers:");
        for (k, v) in &extra_headers {
            println!("  {k}: {v}");
        }
    }
    println!();

    // Build ureq agent that doesn't treat status errors as errors
    let config = ureq::config::Config::builder()
        .http_status_as_error(false)
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();
    let agent = ureq::Agent::new_with_config(config);

    let mut replay_findings = Vec::new();
    let (replay_session_id, replay_session) =
        helpers::create_session(&store, &session.target, "replay", "{}")?;

    for finding in &findings {
        let url = &finding.url;
        if url.is_empty() {
            continue;
        }

        let method = finding.method.as_deref().unwrap_or("GET");
        let start = std::time::Instant::now();

        let result = replay_request(&agent, method, url, &extra_headers);

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(resp) => {
                let body_hash = helpers::sha256_hex(&resp.body);
                let header_list: Vec<String> = resp
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect();

                let original_status = finding
                    .description
                    .split("Status: ")
                    .nth(1)
                    .and_then(|s| s.split(|c: char| !c.is_numeric()).next())
                    .unwrap_or("?");

                let status_match = if original_status == "?" {
                    "?"
                } else if original_status.parse::<u16>() == Ok(resp.status) {
                    "MATCH"
                } else {
                    "DIFF"
                };

                println!(
                    "  {} {} [{}] {}ms ({})",
                    method, url, resp.status, duration_ms, status_match
                );

                // Create a finding for the replay result
                let mut rf = Finding::new(replay_session_id, "replay");
                rf.category = Category::Recon;
                rf.url = url.clone();
                rf.method = Some(method.to_string());
                rf.title = format!("{method} {url} [{}] ({status_match})", resp.status);
                rf.severity = if status_match == "DIFF" {
                    Severity::Medium
                } else {
                    Severity::Info
                };
                rf.description = format!(
                    "Status: {} | Length: {} | Headers: {} | Hash: {body_hash}",
                    resp.status,
                    resp.body.len(),
                    header_list.len()
                );
                rf.evidence = Some(resp.body);
                replay_findings.push(rf);
            }
            Err(e) => {
                println!("  {method} {url} [ERROR] {e}");
                let mut rf = Finding::new(replay_session_id, "replay");
                rf.category = Category::Recon;
                rf.url = url.clone();
                rf.method = Some(method.to_string());
                rf.title = format!("{method} {url} [ERROR]");
                rf.severity = Severity::Low;
                rf.description = format!("Error: {e}");
                replay_findings.push(rf);
            }
        }
    }

    // Store replay findings
    for f in &replay_findings {
        store.insert_finding(f)?;
    }
    helpers::complete_session(&store, &replay_session)?;

    println!(
        "\nReplay complete: {} requests, {} findings",
        findings.len(),
        replay_findings.len()
    );

    helpers::output_findings(&replay_findings, cli)?;
    Ok(())
}

/// Response data extracted from an HTTP response.
struct ReplayResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

fn replay_request(
    agent: &ureq::Agent,
    method: &str,
    url: &str,
    extra_headers: &[(String, String)],
) -> Result<ReplayResponse> {
    match method {
        "POST" => {
            let mut r = agent.post(url);
            for (k, v) in extra_headers {
                r = r.header(k, v);
            }
            let resp = r.send_empty().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(extract_response(resp))
        }
        "PUT" => {
            let mut r = agent.put(url);
            for (k, v) in extra_headers {
                r = r.header(k, v);
            }
            let resp = r.send_empty().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(extract_response(resp))
        }
        "PATCH" => {
            let mut r = agent.patch(url);
            for (k, v) in extra_headers {
                r = r.header(k, v);
            }
            let resp = r.send_empty().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(extract_response(resp))
        }
        "DELETE" => {
            let mut r = agent.delete(url);
            for (k, v) in extra_headers {
                r = r.header(k, v);
            }
            let resp = r.call().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(extract_response(resp))
        }
        "HEAD" => {
            let mut r = agent.head(url);
            for (k, v) in extra_headers {
                r = r.header(k, v);
            }
            let resp = r.call().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(extract_response(resp))
        }
        _ => {
            let mut r = agent.get(url);
            for (k, v) in extra_headers {
                r = r.header(k, v);
            }
            let resp = r.call().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(extract_response(resp))
        }
    }
}

fn extract_response(resp: http::Response<ureq::Body>) -> ReplayResponse {
    use std::io::Read;

    let status = resp.status().as_u16();
    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
        .collect();

    let mut body = String::new();
    let _ = resp.into_body().into_reader().read_to_string(&mut body);

    ReplayResponse {
        status,
        headers,
        body,
    }
}
