use anyhow::Result;

use crate::config::AppConfig;
use crate::finding::Finding;
use crate::report;
use crate::session::store::SessionStore;
use crate::session::{Session, SessionStatus};

/// Resolve auth headers from config by profile name.
pub fn resolve_auth_headers(config: &AppConfig, auth_name: Option<&str>) -> Vec<(String, String)> {
    let Some(name) = auth_name else {
        return vec![];
    };
    config
        .auth
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.to_headers())
        .unwrap_or_default()
}

/// Get the netinject home directory, creating it if needed.
pub fn ensure_home_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("cannot determine home directory"))?;
    let dir = std::path::PathBuf::from(home).join(".netinject");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Open the session store from the default location.
pub fn open_session_store(home_dir: &std::path::Path) -> Result<SessionStore> {
    SessionStore::open(&home_dir.join("store.db"))
}

/// Create a new session and persist it.
pub fn create_session(
    store: &SessionStore,
    target: &str,
    pipeline_name: &str,
    config_snapshot: &str,
) -> Result<(uuid::Uuid, Session)> {
    let session_id = uuid::Uuid::new_v4();
    let session = Session {
        id: session_id,
        created_at: chrono::Utc::now(),
        target: target.to_string(),
        pipeline: Some(pipeline_name.to_string()),
        config_snapshot: config_snapshot.to_string(),
        status: SessionStatus::Running,
        duration_ms: None,
    };
    store.create_session(&session)?;
    Ok((session_id, session))
}

/// Finalize a session with completion status and duration.
pub fn complete_session(store: &SessionStore, session: &Session) -> Result<()> {
    let duration = chrono::Utc::now()
        .signed_duration_since(session.created_at)
        .num_milliseconds();
    store.update_session_status(session.id, SessionStatus::Completed, Some(duration))?;
    Ok(())
}

/// Write findings to output (file or stdout) in the requested format.
pub fn output_findings(findings: &[Finding], cli: &crate::cli::args::Cli) -> Result<()> {
    let output = report::write_findings(findings, cli.format)?;

    if let Some(ref path) = cli.output {
        std::fs::write(path, &output)?;
        if !cli.quiet {
            println!("Results written to {path}");
        }
    } else {
        print!("{output}");
    }
    Ok(())
}
