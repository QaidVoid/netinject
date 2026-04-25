use anyhow::{Context, Result};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

/// Result of running a subprocess, capturing stdout as lines (for JSONL parsing).
pub struct CommandOutput {
    pub stdout_lines: Vec<String>,
    pub stderr: String,
    pub success: bool,
    pub duration: Duration,
}

/// Run a subprocess, capture stdout as lines and stderr as a string.
/// Returns an error only if the command cannot be started.
/// Non-zero exit codes do NOT produce errors — check `success` field.
pub fn run_and_capture(cmd: &str, args: &[&str]) -> Result<CommandOutput> {
    let start = std::time::Instant::now();
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to execute '{cmd}'"))?;

    let duration = start.elapsed();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let stdout_lines = stdout
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.trim().is_empty())
        .collect();

    Ok(CommandOutput {
        stdout_lines,
        stderr,
        success: output.status.success(),
        duration,
    })
}

/// Run a subprocess and return its raw output (for version checks etc).
pub fn run_command(cmd: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute '{cmd}'"))?;
    Ok(output)
}

/// Check if a command/binary exists on PATH.
pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Get the version output of a command.
pub fn get_version(cmd: &str, version_flag: &str) -> Option<String> {
    let output = Command::new(cmd).arg(version_flag).output().ok()?;
    let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ver.is_empty() {
        let ver = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Some(ver)
    } else {
        Some(ver)
    }
}
