use anyhow::Result;

use crate::baseline::BaselineSnapshot;
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

/// SHA-256 hex digest of a string.
pub fn sha256_hex(data: &str) -> String {
    use std::fmt::Write;
    let hash = <sha2::Sha256 as sha2::Digest>::digest(data.as_bytes());
    hash.iter().fold(String::new(), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    })
}

/// Best-effort schema hash: for JSON bodies, normalize values to produce a structural hash.
pub fn schema_hash(body: &str) -> String {
    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(body) {
        normalize_json_values(&mut val);
        sha256_hex(&serde_json::to_string(&val).unwrap_or_default())
    } else {
        sha256_hex(body)
    }
}

/// Recursively replace JSON primitive values with null, keeping structure.
pub fn normalize_json_values(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            for v in map.values_mut() {
                normalize_json_values(v);
            }
        }
        serde_json::Value::Array(arr) => {
            if let Some(first) = arr.first_mut() {
                normalize_json_values(first);
            }
            arr.truncate(1);
        }
        serde_json::Value::String(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Bool(_) => {
            *val = serde_json::Value::Null;
        }
        serde_json::Value::Null => {}
    }
}

/// Resolve a session ID that may be a short prefix to a full UUID.
pub fn resolve_session_id(store: &SessionStore, id: &str) -> Option<uuid::Uuid> {
    if let Ok(uuid) = uuid::Uuid::parse_str(id)
        && store.get_session(uuid).is_ok()
    {
        return Some(uuid);
    }

    let sessions = store.list_sessions().ok()?;
    let matches: Vec<_> = sessions
        .iter()
        .filter(|s| s.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        1 => Some(matches[0].id),
        0 => None,
        _ => {
            tracing::warn!("ambiguous session prefix '{id}', matches multiple sessions");
            None
        }
    }
}

/// Resolve a baseline ID (full UUID or short prefix) to a [`BaselineSnapshot`].
pub fn resolve_baseline(store: &SessionStore, id: &str) -> Option<BaselineSnapshot> {
    if let Ok(uuid) = uuid::Uuid::parse_str(id)
        && let Ok(b) = store.get_baseline(uuid)
    {
        return Some(b);
    }

    let baselines = store.list_baselines().ok()?;
    let matches: Vec<_> = baselines
        .into_iter()
        .filter(|b| b.id.to_string().starts_with(id))
        .collect();

    match matches.len() {
        1 => Some(matches.into_iter().next().unwrap()),
        _ => None,
    }
}
