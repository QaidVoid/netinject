use anyhow::Result;
use std::process::Output;

/// Run a subprocess and return its output.
pub fn run_command(cmd: &str, args: &[&str]) -> Result<Output> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to execute '{cmd}': {e}"))?;
    Ok(output)
}

/// Check if a command/binary exists on PATH.
pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Get the version output of a command.
pub fn get_version(cmd: &str, version_flag: &str) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .arg(version_flag)
        .output()
        .ok()?;
    let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ver.is_empty() {
        let ver = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Some(ver)
    } else {
        Some(ver)
    }
}
