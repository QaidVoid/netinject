use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::{Category, Finding, Severity};
use crate::util;

pub struct FfufAdapter;

/// ffuf JSON output structure (one JSON object per matched result).
///
/// Derived from ffuf source: pkg/ffuf/interfaces.go Result struct.
/// The `input` field maps keyword names (e.g., "FUZZ") to base64-encoded byte values.
/// The `duration` field is in nanoseconds (Go's time.Duration).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FfufResult {
    #[serde(default)]
    input: HashMap<String, String>,
    #[serde(default)]
    position: u64,
    #[serde(default, rename = "status")]
    status_code: i64,
    #[serde(default, rename = "length")]
    content_length: i64,
    #[serde(default, rename = "words")]
    content_words: i64,
    #[serde(default, rename = "lines")]
    content_lines: i64,
    #[serde(default, rename = "content-type")]
    content_type: String,
    #[serde(default)]
    redirectlocation: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    duration: i64,
    #[serde(default)]
    scraper: HashMap<String, Vec<String>>,
    #[serde(default)]
    resultfile: String,
    #[serde(default)]
    host: String,
}

#[async_trait]
impl Adapter for FfufAdapter {
    fn name(&self) -> &str {
        "ffuf"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(util::command_exists("ffuf"))
    }

    fn version(&self) -> Result<String> {
        let output = util::run_command("ffuf", &["-V"])?;
        let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if ver.is_empty() {
            Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
        } else {
            Ok(ver)
        }
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.ffuf;
        let mut all_findings = Vec::new();

        for url in &urls {
            // Build ffuf command args
            let mut args: Vec<String> = vec![
                "-u".into(),
                url.clone(),
                "-w".into(),
                config.wordlist.clone(),
                "-t".into(),
                config.threads.to_string(),
                "-timeout".into(),
                config.timeout.to_string(),
                "-json".into(),
                "-s".into(), // silent mode
                "-noninteractive".into(),
            ];

            if config.recursive {
                args.push("-recursion".into());
                args.push("1".into());
            }

            // Inject auth headers
            for (key, value) in &ctx.auth_headers {
                args.push("-H".into());
                args.push(format!("{key}: {value}"));
            }

            // Apply extra args (e.g., custom matchers/filters)
            if let Some(mc) = ctx.extra_args.get("match_codes") {
                args.push("-mc".into());
                args.push(mc.clone());
            }
            if let Some(fs) = ctx.extra_args.get("filter_size") {
                args.push("-fs".into());
                args.push(fs.clone());
            }

            tracing::info!(adapter = "ffuf", url = %url, "running ffuf");

            let output = util::run_and_capture(
                "ffuf",
                &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )?;

            if !output.success && output.stdout_lines.is_empty() {
                tracing::warn!(
                    adapter = "ffuf",
                    stderr = %output.stderr,
                    "ffuf exited with non-zero status"
                );
            }

            let findings = parse_ffuf_output(&output.stdout_lines, ctx.session_id);
            tracing::info!(
                adapter = "ffuf",
                findings = findings.len(),
                url = %url,
                "ffuf completed"
            );

            all_findings.extend(findings);
        }

        Ok(all_findings)
    }
}

/// Decode base64 input values from ffuf output.
/// ffuf encodes input values as base64 since they're []byte in Go.
fn decode_input_value(val: &str) -> String {
    // ffuf may send base64-encoded values. Try decoding, fall back to raw string.
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, val) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => val.to_string(),
    }
}

/// Determine severity based on HTTP status code from a fuzz result.
fn severity_from_status(status: i64) -> Severity {
    match status {
        200..=299 => Severity::Info,   // Interesting but not necessarily a vuln
        300..=399 => Severity::Info,   // Redirect
        401 => Severity::Low,          // Auth endpoint found
        403 => Severity::Low,          // Forbidden but exists
        405 => Severity::Info,         // Method not allowed
        500..=599 => Severity::Medium, // Server errors may indicate issues
        _ => Severity::Info,
    }
}

fn parse_ffuf_output(lines: &[String], session_id: uuid::Uuid) -> Vec<Finding> {
    lines
        .iter()
        .filter_map(|line| {
            let result: FfufResult = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!(line = %line, error = %e, "failed to parse ffuf JSON line");
                    return None;
                }
            };

            // Extract the fuzz input values
            let input_values: Vec<String> = result
                .input
                .iter()
                .map(|(k, v)| format!("{k}: {}", decode_input_value(v)))
                .collect();

            let input_str = input_values.join(", ");

            let mut finding = Finding::new(session_id, "ffuf");
            finding.severity = severity_from_status(result.status_code);
            finding.category = Category::Fuzz;
            finding.url = result.url.clone();
            finding.method = None; // ffuf doesn't report method in JSON output

            finding.title = if input_str.is_empty() {
                format!("Fuzz hit: {} [{}]", result.url, result.status_code)
            } else {
                format!(
                    "Fuzz hit [{}]: {} [{}]",
                    input_str, result.url, result.status_code
                )
            };

            let mut desc_parts = Vec::new();
            desc_parts.push(format!("Status: {}", result.status_code));
            desc_parts.push(format!("Length: {}", result.content_length));
            desc_parts.push(format!("Words: {}", result.content_words));
            desc_parts.push(format!("Lines: {}", result.content_lines));
            if !result.content_type.is_empty() {
                desc_parts.push(format!("Content-Type: {}", result.content_type));
            }
            if !result.redirectlocation.is_empty() {
                desc_parts.push(format!("Redirect: {}", result.redirectlocation));
            }
            if result.duration > 0 {
                desc_parts.push(format!("Duration: {}ms", result.duration / 1_000_000));
            }
            if !input_str.is_empty() {
                desc_parts.push(format!("Input: {input_str}"));
            }

            finding.description = desc_parts.join(" | ");

            // Evidence is the full response metadata
            finding.evidence = Some(line.clone());

            Some(finding)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ffuf_output() {
        let session_id = uuid::Uuid::new_v4();
        // Base64 of "admin" is "YWRtaW4="
        let lines = vec![
            r#"{"input":{"FUZZ":"YWRtaW4="},"position":42,"status":200,"length":1234,"words":56,"lines":12,"content-type":"text/html","redirectlocation":"","url":"https://example.com/admin","duration":150000000,"scraper":{},"resultfile":"","host":"example.com"}"#.to_string(),
        ];

        let findings = parse_ffuf_output(&lines, session_id);
        assert_eq!(findings.len(), 1);

        let f = &findings[0];
        assert_eq!(f.source, "ffuf");
        assert_eq!(f.category, Category::Fuzz);
        assert_eq!(f.url, "https://example.com/admin");
        assert!(f.title.contains("admin"));
        assert!(f.title.contains("200"));
        assert!(f.description.contains("Length: 1234"));
        assert!(f.description.contains("Words: 56"));
        assert!(f.description.contains("Content-Type: text/html"));
        assert!(f.description.contains("Duration: 150ms"));
        assert!(f.evidence.is_some());
    }

    #[test]
    fn test_parse_ffuf_multiple_results() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"input":{"FUZZ":"YWRtaW4="},"position":1,"status":200,"length":100,"words":10,"lines":5,"content-type":"text/html","redirectlocation":"","url":"https://example.com/admin","duration":50000000,"scraper":{},"resultfile":"","host":"example.com"}"#.to_string(),
            r#"{"input":{"FUZZ":"c2VjcmV0"},"position":2,"status":403,"length":50,"words":5,"lines":2,"content-type":"","redirectlocation":"","url":"https://example.com/secret","duration":30000000,"scraper":{},"resultfile":"","host":"example.com"}"#.to_string(),
            r#"{"input":{"FUZZ":"ZXJyb3I="},"position":3,"status":500,"length":200,"words":20,"lines":8,"content-type":"application/json","redirectlocation":"","url":"https://example.com/error","duration":100000000,"scraper":{},"resultfile":"","host":"example.com"}"#.to_string(),
        ];

        let findings = parse_ffuf_output(&lines, session_id);
        assert_eq!(findings.len(), 3);

        assert_eq!(findings[0].severity, Severity::Info); // 200
        assert_eq!(findings[1].severity, Severity::Low); // 403
        assert_eq!(findings[2].severity, Severity::Medium); // 500
    }

    #[test]
    fn test_parse_ffuf_redirect() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"input":{"FUZZ":"cmVkaXI="},"position":5,"status":301,"length":0,"words":0,"lines":0,"content-type":"","redirectlocation":"/dashboard","url":"https://example.com/redirect","duration":10000000,"scraper":{},"resultfile":"","host":"example.com"}"#.to_string(),
        ];

        let findings = parse_ffuf_output(&lines, session_id);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].description.contains("Redirect: /dashboard"));
    }

    #[test]
    fn test_parse_ffuf_empty() {
        let session_id = uuid::Uuid::new_v4();
        let findings = parse_ffuf_output(&[], session_id);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_ffuf_invalid_json() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec!["not json".to_string()];
        let findings = parse_ffuf_output(&lines, session_id);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_decode_input_value_base64() {
        assert_eq!(decode_input_value("YWRtaW4="), "admin");
        assert_eq!(decode_input_value("c2VjcmV0"), "secret");
    }

    #[test]
    fn test_decode_input_value_plain() {
        // Non-base64 strings fall back to raw
        assert_eq!(decode_input_value("plain_value"), "plain_value");
    }

    #[test]
    fn test_severity_from_status() {
        assert_eq!(severity_from_status(200), Severity::Info);
        assert_eq!(severity_from_status(301), Severity::Info);
        assert_eq!(severity_from_status(401), Severity::Low);
        assert_eq!(severity_from_status(403), Severity::Low);
        assert_eq!(severity_from_status(500), Severity::Medium);
        assert_eq!(severity_from_status(502), Severity::Medium);
    }
}
