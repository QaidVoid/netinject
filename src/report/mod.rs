pub mod jsonl;
pub mod markdown;
pub mod sarif;
pub mod table;

use anyhow::Result;

use crate::finding::Finding;

/// Output format for reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Jsonl,
    Markdown,
    Sarif,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Jsonl => write!(f, "jsonl"),
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Write findings in the requested format.
pub fn write_findings(findings: &[Finding], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Table => table::format(findings),
        OutputFormat::Jsonl => jsonl::format(findings),
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(findings)?;
            Ok(json)
        }
        OutputFormat::Markdown => markdown::format(findings),
        OutputFormat::Sarif => sarif::format(findings),
    }
}
