use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::{Category, Finding, Severity};

pub struct SqlmapAdapter;

#[async_trait]
impl Adapter for SqlmapAdapter {
    fn name(&self) -> &str {
        "sqlmap"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(which::which("sqlmap").is_ok())
    }

    fn version(&self) -> Result<String> {
        let output = std::process::Command::new("sqlmap")
            .arg("--version")
            .output()
            .context("failed to run sqlmap --version")?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.sqlmap;
        let mut findings = Vec::new();

        for url in &urls {
            let url_findings = run_sqlmap(url, ctx).await?;
            findings.extend(url_findings);
        }

        let _ = config;
        Ok(findings)
    }
}

/// Run sqlmap against a single URL and parse output for injection findings.
async fn run_sqlmap(url: &str, ctx: &RunContext) -> Result<Vec<Finding>> {
    let config = &ctx.config.adapters.sqlmap;

    // Create a temp dir for sqlmap output
    let tmp_dir = tempfile::tempdir()?;
    let output_dir = tmp_dir.path().to_string_lossy().to_string();

    let mut args = vec![
        "-u".to_string(),
        url.to_string(),
        "--batch".to_string(),
        "--output-dir".to_string(),
        output_dir,
        format!("--level={}", config.level),
        format!("--risk={}", config.risk),
    ];

    // Add auth headers
    for (key, value) in &ctx.auth_headers {
        args.push("--headers".to_string());
        args.push(format!("{key}: {value}"));
    }

    tracing::info!(adapter = "sqlmap", url = %url, "running sqlmap");

    let output = crate::util::run_and_capture(
        "sqlmap",
        &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    )?;

    if !output.success {
        tracing::warn!(adapter = "sqlmap", url = %url, "sqlmap exited with non-zero status");
    }

    let findings = parse_sqlmap_output(&output.stdout_lines, ctx.session_id, url);

    tracing::info!(
        adapter = "sqlmap",
        url = %url,
        findings = findings.len(),
        "sqlmap completed"
    );

    Ok(findings)
}

/// Parse sqlmap stdout lines for injection findings.
///
/// sqlmap outputs lines like:
///   [hh:mm:ss] [INFO] GET parameter 'id' is vulnerable. Do you want to keep testing ...
///   [hh:mm:ss] [INFO] GET parameter 'id' is 'Boolean-based blind - WHERE ...' injectable
///   [hh:mm:ss] [CRITICAL] connection timed out ...
///   [hh:mm:ss] [WARNING] GET parameter 'id' does not appear to be injectable
fn parse_sqlmap_output(lines: &[String], session_id: uuid::Uuid, url: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for line in lines {
        // Skip negative patterns (e.g. "does not appear to be injectable")
        if line.contains("does not") || line.contains("not appear") {
            continue;
        }

        // Match injection found patterns
        if let Some(finding) = parse_injection_line(line, session_id, url) {
            findings.push(finding);
            continue;
        }

        // Match vulnerability found patterns
        if let Some(finding) = parse_vulnerable_line(line, session_id, url) {
            findings.push(finding);
        }
    }

    findings
}

/// Parse lines like: `... parameter 'id' is 'Boolean-based blind - WHERE ...' injectable`
fn parse_injection_line(line: &str, session_id: uuid::Uuid, url: &str) -> Option<Finding> {
    // Look for "injectable" keyword
    if !line.contains("injectable") {
        return None;
    }

    // Try to extract parameter name and injection type
    let param = extract_quoted(line, 0).unwrap_or_else(|| "unknown".to_string());
    let injection_type = extract_quoted(line, 1).unwrap_or_else(|| "SQL injection".to_string());

    let severity = severity_from_injection_type(&injection_type);

    let mut finding = Finding::new(session_id, "sqlmap");
    finding.severity = severity;
    finding.category = Category::Injection;
    finding.title = format!("SQL injection in parameter '{param}'");
    finding.description = format!("Parameter '{param}' is '{injection_type}' injectable");
    finding.url = url.to_string();
    finding.evidence = Some(line.trim().to_string());
    finding.cwe = Some("CWE-89".to_string()); // SQL Injection
    Some(finding)
}

/// Parse lines like: `... parameter 'id' is vulnerable`
fn parse_vulnerable_line(line: &str, session_id: uuid::Uuid, url: &str) -> Option<Finding> {
    if !line.contains("is vulnerable") {
        return None;
    }

    let param = extract_quoted(line, 0).unwrap_or_else(|| "unknown".to_string());

    let mut finding = Finding::new(session_id, "sqlmap");
    finding.severity = Severity::Critical;
    finding.category = Category::Injection;
    finding.title = format!("SQL injection in parameter '{param}'");
    finding.description = format!("Parameter '{param}' is vulnerable to SQL injection");
    finding.url = url.to_string();
    finding.evidence = Some(line.trim().to_string());
    finding.cwe = Some("CWE-89".to_string());
    Some(finding)
}

/// Extract the Nth quoted string from a line (single-quoted).
fn extract_quoted(line: &str, nth: usize) -> Option<String> {
    let mut count = 0;
    let mut chars = line.chars();
    let mut result = String::new();
    let mut in_quote = false;

    for c in chars.by_ref() {
        if c == '\'' {
            if in_quote {
                if count == nth {
                    return Some(result);
                }
                count += 1;
                result.clear();
                in_quote = false;
            } else {
                in_quote = true;
                result.clear();
            }
        } else if in_quote {
            result.push(c);
        }
    }
    None
}

/// Determine severity based on injection type string.
fn severity_from_injection_type(injection_type: &str) -> Severity {
    let lower = injection_type.to_lowercase();
    if lower.contains("error-based") || lower.contains("union") {
        Severity::Critical
    } else if lower.contains("boolean-based") || lower.contains("time-based") {
        Severity::High
    } else if lower.contains("stacked") || lower.contains("inline") {
        Severity::Critical
    } else {
        Severity::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_injectable_line() {
        let session_id = uuid::Uuid::new_v4();
        let line =
            "[12:34:56] [INFO] GET parameter 'id' is 'Boolean-based blind - WHERE ...' injectable";
        let finding =
            parse_injection_line(line, session_id, "https://example.com/page?id=1").unwrap();

        assert_eq!(finding.source, "sqlmap");
        assert_eq!(finding.category, Category::Injection);
        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("id"));
        assert_eq!(finding.url, "https://example.com/page?id=1");
        assert_eq!(finding.cwe.as_deref(), Some("CWE-89"));
    }

    #[test]
    fn test_parse_error_based_injection() {
        let session_id = uuid::Uuid::new_v4();
        let line = "[12:34:56] [INFO] POST parameter 'username' is 'MySQL >= 5.0 AND error-based' injectable";
        let finding = parse_injection_line(line, session_id, "https://example.com/login").unwrap();

        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.title.contains("username"));
    }

    #[test]
    fn test_parse_vulnerable_line() {
        let session_id = uuid::Uuid::new_v4();
        let line =
            "[12:34:56] [INFO] GET parameter 'id' is vulnerable. Do you want to keep testing...";
        let finding =
            parse_vulnerable_line(line, session_id, "https://example.com/page?id=1").unwrap();

        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.description.contains("id"));
    }

    #[test]
    fn test_parse_no_injection() {
        let lines = vec![
            "[12:34:56] [INFO] testing connection to the target URL".into(),
            "[12:34:56] [WARNING] GET parameter 'id' does not appear to be injectable".into(),
        ];
        let findings = parse_sqlmap_output(&lines, uuid::Uuid::new_v4(), "https://example.com");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_multiple_injections() {
        let lines = vec![
            "[12:34:56] [INFO] GET parameter 'id' is 'Boolean-based blind' injectable".into(),
            "[12:34:56] [INFO] GET parameter 'id' is 'UNION query' injectable".into(),
        ];
        let findings = parse_sqlmap_output(&lines, uuid::Uuid::new_v4(), "https://example.com");
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn test_extract_quoted() {
        let line = "parameter 'foo' is 'bar baz' injectable";
        assert_eq!(extract_quoted(line, 0), Some("foo".to_string()));
        assert_eq!(extract_quoted(line, 1), Some("bar baz".to_string()));
        assert_eq!(extract_quoted(line, 2), None);
    }

    #[test]
    fn test_severity_from_injection_type() {
        assert_eq!(
            severity_from_injection_type("Boolean-based blind"),
            Severity::High
        );
        assert_eq!(
            severity_from_injection_type("Time-based blind"),
            Severity::High
        );
        assert_eq!(
            severity_from_injection_type("UNION query"),
            Severity::Critical
        );
        assert_eq!(
            severity_from_injection_type("Error-based"),
            Severity::Critical
        );
        assert_eq!(
            severity_from_injection_type("Stacked queries"),
            Severity::Critical
        );
        assert_eq!(
            severity_from_injection_type("Generic SQL injection"),
            Severity::High
        );
    }
}
