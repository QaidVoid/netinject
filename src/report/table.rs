use anyhow::Result;

use crate::finding::Finding;

pub fn format(findings: &[Finding]) -> Result<String> {
    if findings.is_empty() {
        return Ok("No findings.".into());
    }

    let rows: Vec<(&str, &str, &str, &str)> = findings
        .iter()
        .map(|f| {
            (
                f.severity.to_string().leak() as &str,
                f.source.as_str(),
                f.title.as_str(),
                f.url.as_str(),
            )
        })
        .collect();

    let mut table = tabled::Table::new(rows);
    table.with(tabled::settings::Style::modern());

    Ok(format!("{table}\n"))
}
