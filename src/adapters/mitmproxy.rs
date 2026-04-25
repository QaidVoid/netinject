use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io::Write;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::{Category, Finding, Severity};

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

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.mitmproxy;

        // Write URLs to a temp file for replay through mitmdump
        let tmp_dir = tempfile::tempdir()?;
        let urls_file = tmp_dir.path().join("urls.txt");
        let mut f = std::fs::File::create(&urls_file)?;
        for url in &urls {
            writeln!(f, "{url}")?;
        }

        // Write an inline mitmdump script that logs request/response summaries
        let script_path = tmp_dir.path().join("netinject_addon.py");
        write_mitmdump_script(&script_path)?;

        let dump_file = tmp_dir.path().join("flows.txt");

        let listen_addr = format!("{}:{}", config.listen_host, config.listen_port);
        let port_str = config.listen_port.to_string();
        let script_str = script_path.to_string_lossy().to_string();
        let dump_str = format!("dumpfile={}", dump_file.to_string_lossy());

        let args: Vec<&str> = vec![
            "--listen-host",
            &config.listen_host,
            "--listen-port",
            &port_str,
            "-s",
            &script_str,
            "--set",
            &dump_str,
            "-q",
        ];

        tracing::info!(
            adapter = "mitmproxy",
            listen = %listen_addr,
            "starting mitmdump to capture traffic"
        );

        // Run mitmdump — it will run until killed or the script exits.
        // For now, we use it in "replay" mode: feed URLs through it.
        // In practice, mitmproxy is used as a proxy that other adapters route through.
        let output = crate::util::run_and_capture("mitmdump", &args)?;

        // Parse the dump file for findings
        let findings = if dump_file.exists() {
            let content = std::fs::read_to_string(&dump_file)?;
            parse_flow_dump(&content, ctx.session_id)
        } else {
            // Parse stdout for inline findings from the addon script
            parse_addon_output(&output.stdout_lines, ctx.session_id)
        };

        tracing::info!(
            adapter = "mitmproxy",
            findings = findings.len(),
            "mitmdump completed"
        );

        Ok(findings)
    }
}

/// Write a mitmdump addon script that captures request/response info and prints JSON.
fn write_mitmdump_script(path: &std::path::Path) -> Result<()> {
    let script = r#"""
import json
from mitmproxy import http

def response(flow: http.HTTPFlow):
    finding = {
        "url": flow.request.pretty_url,
        "method": flow.request.method,
        "status_code": flow.response.status_code if flow.response else 0,
        "content_type": flow.response.headers.get("content-type", "") if flow.response else "",
        "content_length": len(flow.response.content) if flow.response and flow.response.content else 0,
    }
    # Check for sensitive data patterns
    issues = []
    if flow.response and flow.response.content:
        body = flow.response.content.decode("utf-8", errors="replace")
        # Check for tokens/keys in response body
        for pattern_name, pattern in [
            ("AWS Access Key", r"AKIA[0-9A-Z]{16}"),
            ("AWS Secret Key", r"(?i)aws_secret_access_key"),
            ("Generic Secret", r"(?i)(password|secret|token|api.key)\s*[:=]\s*['\"]?[\w\-]{8,}"),
            ("Private Key", r"-----BEGIN (RSA |EC |DSA )?PRIVATE KEY-----"),
            ("Credit Card", r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b"),
        ]:
            import re
            if re.search(pattern, body):
                issues.append(pattern_name)
        # Check for auth tokens in URL
        import re
        url = flow.request.pretty_url
        for param_name, pattern in [
            ("token in URL", r"[?&](token|access_token|auth)=[^&]+"),
            ("API key in URL", r"[?&](api_key|apikey|key)=[^&]+"),
        ]:
            if re.search(pattern, url, re.IGNORECASE):
                issues.append(param_name)
    if issues:
        finding["issues"] = issues
        finding["sensitive_data_found"] = True
    else:
        finding["sensitive_data_found"] = False
    print(json.dumps(finding))
"""#;
    std::fs::write(path, script)?;
    Ok(())
}

/// Parse JSON lines from the addon script output.
fn parse_addon_output(lines: &[String], session_id: uuid::Uuid) -> Vec<Finding> {
    let mut findings = Vec::new();

    for line in lines {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line)
            && val.get("sensitive_data_found").and_then(|v| v.as_bool()) == Some(true)
        {
            let url = val.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let issues = val
                .get("issues")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();

            let mut finding = Finding::new(session_id, "mitmproxy");
            finding.severity = Severity::High;
            finding.category = Category::DataExposure;
            finding.title = format!("Sensitive data detected: {issues}");
            finding.description =
                format!("mitmproxy detected sensitive data patterns in traffic to {url}: {issues}");
            finding.url = url.to_string();
            finding.method = val
                .get("method")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            finding.evidence = Some(line.clone());
            findings.push(finding);
        }
    }

    findings
}

/// Parse a flow dump file for interesting patterns.
fn parse_flow_dump(content: &str, session_id: uuid::Uuid) -> Vec<Finding> {
    // The dump file is written by our addon script as JSONL
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    parse_addon_output(&lines, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addon_output_with_sensitive_data() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"url":"https://api.example.com/data","method":"GET","status_code":200,"content_type":"application/json","content_length":123,"sensitive_data_found":true,"issues":["AWS Access Key","token in URL"]}"#.to_string(),
            r#"{"url":"https://api.example.com/safe","method":"GET","status_code":200,"content_type":"application/json","content_length":50,"sensitive_data_found":false}"#.to_string(),
        ];

        let findings = parse_addon_output(&lines, session_id);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].source, "mitmproxy");
        assert_eq!(findings[0].category, Category::DataExposure);
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].title.contains("AWS Access Key"));
        assert!(findings[0].title.contains("token in URL"));
    }

    #[test]
    fn test_parse_addon_output_no_findings() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"url":"https://api.example.com/safe","method":"GET","status_code":200,"sensitive_data_found":false}"#.to_string(),
        ];

        let findings = parse_addon_output(&lines, session_id);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_addon_output_invalid_json() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            "not json".to_string(),
            r#"{"url":"https://example.com","method":"GET","status_code":200,"sensitive_data_found":false}"#.to_string(),
        ];

        let findings = parse_addon_output(&lines, session_id);
        assert!(findings.is_empty());
    }
}
