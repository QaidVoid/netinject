use serde::{Deserialize, Serialize};

/// A captured response snapshot for one endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSnapshot {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub target: String,
    pub spec_hash: String,
    pub entries: Vec<BaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body_hash: String,
    pub body_schema_hash: Option<String>,
    pub response_time_ms: u64,
}

/// Result of comparing current state against a baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    pub baseline_id: uuid::Uuid,
    pub checked_at: chrono::DateTime<chrono::Utc>,
    pub target: String,
    pub diffs: Vec<EndpointDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDiff {
    pub method: String,
    pub path: String,
    pub changes: Vec<Change>,
    pub severity: DiffSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Change {
    StatusCodeChanged { from: u16, to: u16 },
    HeaderAdded(String),
    HeaderRemoved(String),
    SchemaDrift { description: String },
    TimingAnomaly { baseline_ms: u64, current_ms: u64 },
    BodyChanged { from_hash: String, to_hash: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffSeverity {
    Breaking,
    Warning,
    Info,
}
