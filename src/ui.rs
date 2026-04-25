//! Terminal UI helpers for colored, styled output.
//!
//! Respects `--no-color` flag and `NO_COLOR` environment variable.

use console::Style;
use std::time::Duration;

use crate::finding::{Category, Severity};

// ── Color gate ──────────────────────────────────────────────────────────────

/// Returns true if colors should be used (respects --no-color and NO_COLOR env).
pub fn colors_enabled(cli_no_color: bool) -> bool {
    if cli_no_color {
        return false;
    }
    std::env::var("NO_COLOR").is_err() && std::env::var("TERM").as_deref() != Ok("dumb")
}

// ── Styled primitives ───────────────────────────────────────────────────────

pub fn success(text: &str) -> String {
    Style::new().green().bold().apply_to(text).to_string()
}

pub fn error(text: &str) -> String {
    Style::new().red().bold().apply_to(text).to_string()
}

pub fn warning(text: &str) -> String {
    Style::new().yellow().bold().apply_to(text).to_string()
}

pub fn dim(text: &str) -> String {
    Style::new().dim().apply_to(text).to_string()
}

pub fn bold(text: &str) -> String {
    Style::new().bold().apply_to(text).to_string()
}

pub fn info(text: &str) -> String {
    Style::new().cyan().apply_to(text).to_string()
}

// ── Severity styling ────────────────────────────────────────────────────────

/// Format a severity level with appropriate color.
pub fn styled_severity(severity: Severity) -> String {
    match severity {
        Severity::Critical => Style::new().red().bold().apply_to("CRITICAL").to_string(),
        Severity::High => Style::new().red().apply_to("HIGH").to_string(),
        Severity::Medium => Style::new().yellow().apply_to("MEDIUM").to_string(),
        Severity::Low => Style::new().blue().apply_to("LOW").to_string(),
        Severity::Info => Style::new().dim().apply_to("INFO").to_string(),
    }
}

/// Style for diff severity labels.
pub fn styled_diff_severity(label: &str) -> String {
    match label {
        "BREAKING" => Style::new().red().bold().apply_to(label).to_string(),
        "WARNING" => Style::new().yellow().bold().apply_to(label).to_string(),
        "INFO" => Style::new().cyan().apply_to(label).to_string(),
        _ => label.to_string(),
    }
}

// ── Diff prefix styling ─────────────────────────────────────────────────────

pub fn diff_added(text: &str) -> String {
    Style::new().green().apply_to(text).to_string()
}

pub fn diff_removed(text: &str) -> String {
    Style::new().red().apply_to(text).to_string()
}

pub fn diff_changed(text: &str) -> String {
    Style::new().yellow().apply_to(text).to_string()
}

// ── Check mark / cross ──────────────────────────────────────────────────────

pub fn check_mark() -> String {
    Style::new().green().apply_to("✓").to_string()
}

pub fn cross_mark() -> String {
    Style::new().red().apply_to("✗").to_string()
}

// ── Category styling ────────────────────────────────────────────────────────

pub fn styled_category(cat: Category) -> String {
    let s = match cat {
        Category::Injection => Style::new().red().apply_to(cat.to_string()),
        Category::Auth => Style::new().yellow().apply_to(cat.to_string()),
        Category::DataExposure => Style::new().magenta().apply_to(cat.to_string()),
        Category::Misconfig => Style::new().cyan().apply_to(cat.to_string()),
        Category::Regression => Style::new().red().bold().apply_to(cat.to_string()),
        Category::Fuzz => Style::new().blue().apply_to(cat.to_string()),
        Category::Recon => Style::new().green().apply_to(cat.to_string()),
        Category::Unknown => Style::new().dim().apply_to(cat.to_string()),
    };
    s.to_string()
}

// ── Banners ─────────────────────────────────────────────────────────────────

/// Print a section header with a styled label.
pub fn section_header(label: &str) {
    println!();
    println!("{}", Style::new().bold().apply_to(label));
}

/// Print a summary line showing finding counts by severity.
pub fn print_finding_summary(findings: &[crate::finding::Finding]) {
    if findings.is_empty() {
        println!("\n{}", dim("No findings."));
        return;
    }

    let mut counts: std::collections::HashMap<Severity, usize> = std::collections::HashMap::new();
    for f in findings {
        *counts.entry(f.severity).or_default() += 1;
    }

    let total = findings.len();
    let parts: Vec<String> = [
        (Severity::Critical, "critical"),
        (Severity::High, "high"),
        (Severity::Medium, "medium"),
        (Severity::Low, "low"),
        (Severity::Info, "info"),
    ]
    .iter()
    .filter_map(|(sev, label)| {
        counts.get(sev).map(|&c| {
            let styled = Style::new()
                .bold()
                .fg(match sev {
                    Severity::Critical | Severity::High => console::Color::Red,
                    Severity::Medium => console::Color::Yellow,
                    Severity::Low => console::Color::Blue,
                    Severity::Info => console::Color::White,
                })
                .apply_to(format!("{c} {label}"));
            styled.to_string()
        })
    })
    .collect();

    println!(
        "\n{} {} {}: {}",
        Style::new().bold().apply_to("Summary"),
        Style::new().bold().apply_to(total),
        if total == 1 { "finding" } else { "findings" },
        parts.join(", ")
    );
}

/// Format a duration for display.
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs < 1.0 {
        format!("{:.0}ms", d.as_millis())
    } else {
        format!("{:.1}s", secs)
    }
}
