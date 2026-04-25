use anyhow::Result;

use crate::finding::Finding;

pub fn format(findings: &[Finding]) -> Result<String> {
    let mut report = String::from("# netinject Report\n\n");
    report.push_str(&format!("**Findings:** {}\n\n", findings.len()));

    for f in findings {
        report.push_str(&format!(
            "### {} [{}]\n\n- **Severity:** {}\n- **Category:** {}\n- **URL:** {}\n",
            f.title, f.source, f.severity, f.category, f.url
        ));
        if !f.description.is_empty() {
            report.push_str(&format!("- **Description:** {}\n", f.description));
        }
        if let Some(ref cwe) = f.cwe {
            report.push_str(&format!("- **CWE:** {cwe}\n"));
        }
        report.push('\n');
    }

    Ok(report)
}
