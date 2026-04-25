use anyhow::Result;

use super::{Session, SessionStatus};
use crate::finding::Finding;

/// SQLite-backed session store.
pub struct SessionStore {
    conn: rusqlite::Connection,
}

impl SessionStore {
    /// Open (or create) the session store at the given path.
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory store (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                created_at  TEXT NOT NULL,
                target      TEXT NOT NULL,
                pipeline    TEXT,
                config      TEXT NOT NULL,
                status      TEXT NOT NULL,
                duration_ms INTEGER
            );

            CREATE TABLE IF NOT EXISTS findings (
                id          TEXT PRIMARY KEY,
                session_id  TEXT NOT NULL REFERENCES sessions(id),
                source      TEXT NOT NULL,
                severity    TEXT NOT NULL,
                category    TEXT NOT NULL,
                title       TEXT NOT NULL,
                description TEXT,
                url         TEXT NOT NULL,
                method      TEXT,
                evidence    TEXT,
                cwe         TEXT,
                reference   TEXT,
                created_at  TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_findings_session ON findings(session_id);
            CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity);
            CREATE INDEX IF NOT EXISTS idx_findings_source ON findings(source);
            CREATE INDEX IF NOT EXISTS idx_findings_url ON findings(url);

            CREATE TABLE IF NOT EXISTS baselines (
                id          TEXT PRIMARY KEY,
                created_at  TEXT NOT NULL,
                target      TEXT NOT NULL,
                spec_hash   TEXT NOT NULL,
                snapshot    TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    /// Create a new session.
    pub fn create_session(&self, session: &Session) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, created_at, target, pipeline, config, status, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                session.id.to_string(),
                session.created_at.to_rfc3339(),
                &session.target,
                &session.pipeline,
                &session.config_snapshot,
                session.status.to_string(),
                session.duration_ms,
            ),
        )?;
        Ok(())
    }

    /// Update session status.
    pub fn update_session_status(
        &self,
        id: uuid::Uuid,
        status: SessionStatus,
        duration_ms: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET status = ?1, duration_ms = ?2 WHERE id = ?3",
            (status.to_string(), duration_ms, id.to_string()),
        )?;
        Ok(())
    }

    /// Store a finding.
    pub fn insert_finding(&self, finding: &Finding) -> Result<()> {
        self.conn.execute(
            "INSERT INTO findings (id, session_id, source, severity, category, title, description, url, method, evidence, cwe, reference, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            (
                finding.id.to_string(),
                finding.session_id.to_string(),
                &finding.source,
                finding.severity.to_string(),
                finding.category.to_string(),
                &finding.title,
                &finding.description,
                &finding.url,
                &finding.method,
                &finding.evidence,
                &finding.cwe,
                &finding.reference,
                finding.timestamp.to_rfc3339(),
            ),
        )?;
        Ok(())
    }

    /// List all sessions, newest first.
    pub fn list_sessions(&self) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, target, pipeline, config, status, duration_ms
             FROM sessions ORDER BY created_at DESC",
        )?;
        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: uuid::Uuid::parse_str(row.get::<_, String>(0).unwrap().as_str()).unwrap(),
                    created_at: row.get::<_, String>(1).unwrap().parse().unwrap(),
                    target: row.get(2).unwrap(),
                    pipeline: row.get(3).unwrap(),
                    config_snapshot: row.get(4).unwrap(),
                    status: match row.get::<_, String>(5).unwrap().as_str() {
                        "running" => SessionStatus::Running,
                        "completed" => SessionStatus::Completed,
                        "failed" => SessionStatus::Failed,
                        _ => SessionStatus::Failed,
                    },
                    duration_ms: row.get(6).unwrap(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    /// Get all findings for a session.
    pub fn get_findings(&self, session_id: uuid::Uuid) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, source, severity, category, title, description, url, method, evidence, cwe, reference, created_at
             FROM findings WHERE session_id = ?1",
        )?;
        let findings = stmt
            .query_map([session_id.to_string()], |row| {
                let severity_str: String = row.get(3).unwrap();
                let category_str: String = row.get(4).unwrap();
                Ok(Finding {
                    id: uuid::Uuid::parse_str(row.get::<_, String>(0).unwrap().as_str()).unwrap(),
                    session_id: uuid::Uuid::parse_str(row.get::<_, String>(1).unwrap().as_str())
                        .unwrap(),
                    source: row.get(2).unwrap(),
                    severity: parse_severity(&severity_str),
                    category: parse_category(&category_str),
                    title: row.get(5).unwrap(),
                    description: row.get::<_, Option<String>>(6).unwrap().unwrap_or_default(),
                    url: row.get(7).unwrap(),
                    method: row.get(8).unwrap(),
                    evidence: row.get(9).unwrap(),
                    cwe: row.get(10).unwrap(),
                    reference: row.get(11).unwrap(),
                    timestamp: row.get::<_, String>(12).unwrap().parse().unwrap(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(findings)
    }
}

fn parse_severity(s: &str) -> crate::finding::Severity {
    match s.to_uppercase().as_str() {
        "CRITICAL" => crate::finding::Severity::Critical,
        "HIGH" => crate::finding::Severity::High,
        "MEDIUM" => crate::finding::Severity::Medium,
        "LOW" => crate::finding::Severity::Low,
        _ => crate::finding::Severity::Info,
    }
}

fn parse_category(s: &str) -> crate::finding::Category {
    match s {
        "injection" => crate::finding::Category::Injection,
        "auth" => crate::finding::Category::Auth,
        "data_exposure" => crate::finding::Category::DataExposure,
        "misconfiguration" => crate::finding::Category::Misconfig,
        "regression" => crate::finding::Category::Regression,
        "fuzz" => crate::finding::Category::Fuzz,
        "recon" => crate::finding::Category::Recon,
        _ => crate::finding::Category::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list_sessions() {
        let store = SessionStore::open_in_memory().unwrap();
        let session = Session {
            id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            target: "https://example.com".into(),
            pipeline: Some("full-api-scan".into()),
            config_snapshot: "{}".into(),
            status: SessionStatus::Running,
            duration_ms: None,
        };
        store.create_session(&session).unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].target, "https://example.com");
    }

    #[test]
    fn test_insert_and_get_findings() {
        let store = SessionStore::open_in_memory().unwrap();
        let session_id = uuid::Uuid::new_v4();
        let session = Session {
            id: session_id,
            created_at: chrono::Utc::now(),
            target: "https://example.com".into(),
            pipeline: None,
            config_snapshot: "{}".into(),
            status: SessionStatus::Completed,
            duration_ms: Some(5000),
        };
        store.create_session(&session).unwrap();

        let finding = Finding::new(session_id, "nuclei");
        store.insert_finding(&finding).unwrap();

        let findings = store.get_findings(session_id).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].source, "nuclei");
    }
}
