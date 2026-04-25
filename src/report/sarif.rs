use anyhow::Result;

use crate::finding::Finding;

pub fn format(findings: &[Finding]) -> Result<String> {
    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            let level = match f.severity {
                crate::finding::Severity::Critical | crate::finding::Severity::High => "error",
                crate::finding::Severity::Medium => "warning",
                _ => "note",
            };
            serde_json::json!({
                "ruleId": f.source,
                "level": level,
                "message": {
                    "text": format!("{}: {}", f.title, f.description)
                },
                "locations": [{
                    "physicalLocation": {
                        "uri": f.url
                    }
                }]
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "netinject",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/QaidVoid/netinject",
                    "rules": []
                }
            },
            "results": results
        }]
    });
    Ok(serde_json::to_string_pretty(&sarif)?)
}
