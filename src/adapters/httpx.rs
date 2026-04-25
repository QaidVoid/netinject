use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::{Category, Finding, Severity};
use crate::util;

pub struct HttpxAdapter;

/// httpx JSON output structure (one JSON object per line).
/// We only parse the fields we care about.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HttpxResult {
    #[serde(default)]
    url: String,
    #[serde(default)]
    input: String,
    #[serde(default)]
    status_code: u16,
    #[serde(default)]
    content_length: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    webserver: String,
    #[serde(default)]
    tech: Vec<String>,
    #[serde(default)]
    method: String,
    #[serde(default)]
    content_type: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    host_ip: String,
    #[serde(default)]
    failed: bool,
    #[serde(default)]
    time: String,
}

#[async_trait]
impl Adapter for HttpxAdapter {
    fn name(&self) -> &str {
        "httpx"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(util::command_exists("httpx"))
    }

    fn version(&self) -> Result<String> {
        let output = util::run_command("httpx", &["-version"])?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.httpx;

        let mut args: Vec<String> = vec![
            "-json".into(),
            "-silent".into(),
            "-threads".into(),
            config.threads.to_string(),
            "-rate-limit".into(),
            config.rate_limit.to_string(),
            "-sc".into(),     // status code
            "-title".into(),  // page title
            "-server".into(), // web server
            "-method".into(), // request method
            "-content-type".into(),
            "-ip".into(), // host IP
        ];

        if config.tech_detect {
            args.push("-tech-detect".into());
        }

        // Pass URLs via stdin using -l flag with a temp file, or as args
        // httpx reads from stdin by default, but also accepts -u
        for url in &urls {
            args.push("-u".into());
            args.push(url.clone());
        }

        // Inject auth headers
        for (key, value) in &ctx.auth_headers {
            args.push("-H".into());
            args.push(format!("{key}: {value}"));
        }

        tracing::info!(adapter = "httpx", count = urls.len(), "running httpx");

        let output = util::run_and_capture(
            "httpx",
            &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )?;

        if !output.success && output.stdout_lines.is_empty() {
            tracing::warn!(
                adapter = "httpx",
                stderr = %output.stderr,
                "httpx exited with non-zero status"
            );
        }

        let findings = parse_httpx_output(&output.stdout_lines, ctx.session_id);
        tracing::info!(
            adapter = "httpx",
            findings = findings.len(),
            "httpx completed"
        );

        Ok(findings)
    }
}

fn parse_httpx_output(lines: &[String], session_id: uuid::Uuid) -> Vec<Finding> {
    lines
        .iter()
        .filter_map(|line| {
            let result: HttpxResult = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!(line = %line, error = %e, "failed to parse httpx JSON line");
                    return None;
                }
            };

            if result.failed {
                return None;
            }

            let mut finding = Finding::new(session_id, "httpx");
            finding.severity = Severity::Info;
            finding.category = Category::Recon;
            finding.url = result.url.clone();
            finding.method = if result.method.is_empty() {
                None
            } else {
                Some(result.method)
            };

            let title = if result.title.is_empty() {
                format!("Live endpoint: {}", result.url)
            } else {
                format!("Live endpoint [{}]: {}", result.title, result.url)
            };
            finding.title = title;

            let mut desc_parts = Vec::new();
            desc_parts.push(format!("Status: {}", result.status_code));
            desc_parts.push(format!("Content-Length: {}", result.content_length));
            if !result.webserver.is_empty() {
                desc_parts.push(format!("Server: {}", result.webserver));
            }
            if !result.content_type.is_empty() {
                desc_parts.push(format!("Content-Type: {}", result.content_type));
            }
            if !result.host_ip.is_empty() {
                desc_parts.push(format!("IP: {}", result.host_ip));
            }
            if !result.tech.is_empty() {
                desc_parts.push(format!("Technologies: {}", result.tech.join(", ")));
            }
            finding.description = desc_parts.join(" | ");

            Some(finding)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_httpx_output() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"timestamp":"2026-04-25T22:09:39.284Z","port":"443","url":"https://scanme.sh","input":"https://scanme.sh","scheme":"https","content_type":"text/plain","method":"GET","host":"scanme.sh","host_ip":"128.199.158.128","path":"/","time":"611ms","status_code":200,"content_length":2,"failed":false}"#.to_string(),
            r#"{"timestamp":"2026-04-25T22:09:40.100Z","port":"443","url":"https://example.com","input":"https://example.com","scheme":"https","content_type":"text/html","method":"GET","host":"example.com","host_ip":"93.184.216.34","path":"/","time":"200ms","status_code":200,"content_length":1256,"title":"Example Domain","webserver":"ECS (dcb/7F84)","tech":["Nginx"],"failed":false}"#.to_string(),
        ];

        let findings = parse_httpx_output(&lines, session_id);
        assert_eq!(findings.len(), 2);

        assert_eq!(findings[0].url, "https://scanme.sh");
        assert_eq!(findings[0].method, Some("GET".into()));
        assert_eq!(findings[0].source, "httpx");
        assert_eq!(findings[0].category, Category::Recon);
        assert!(findings[0].description.contains("Status: 200"));
        assert!(findings[0].description.contains("IP: 128.199.158.128"));

        assert_eq!(findings[1].url, "https://example.com");
        assert!(findings[1].title.contains("Example Domain"));
        assert!(findings[1].description.contains("Server: ECS (dcb/7F84)"));
        assert!(findings[1].description.contains("Nginx"));
    }

    #[test]
    fn test_parse_httpx_skips_failed() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"url":"https://down.example.com","status_code":0,"failed":true}"#.to_string(),
            r#"{"url":"https://up.example.com","status_code":200,"failed":false,"method":"GET"}"#
                .to_string(),
        ];

        let findings = parse_httpx_output(&lines, session_id);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].url, "https://up.example.com");
    }

    #[test]
    fn test_parse_httpx_invalid_json() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec!["not valid json".to_string()];
        let findings = parse_httpx_output(&lines, session_id);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_httpx_empty() {
        let session_id = uuid::Uuid::new_v4();
        let findings = parse_httpx_output(&[], session_id);
        assert!(findings.is_empty());
    }
}
