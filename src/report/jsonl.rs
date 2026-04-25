use anyhow::Result;

use crate::finding::Finding;

pub fn format(findings: &[Finding]) -> Result<String> {
    let mut lines = Vec::new();
    for f in findings {
        lines.push(serde_json::to_string(f)?);
    }
    Ok(lines.join("\n"))
}
