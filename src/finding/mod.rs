use serde::{Deserialize, Serialize};

/// Normalized security finding from any source (adapter, regression, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub session_id: uuid::Uuid,
    pub source: String,
    pub severity: Severity,
    pub category: Category,
    pub title: String,
    pub description: String,
    pub url: String,
    pub method: Option<String>,
    pub evidence: Option<String>,
    pub cwe: Option<String>,
    pub reference: Option<String>,
}

impl Finding {
    pub fn new(session_id: uuid::Uuid, source: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            session_id,
            source: source.into(),
            severity: Severity::Info,
            category: Category::Unknown,
            title: String::new(),
            description: String::new(),
            url: String::new(),
            method: None,
            evidence: None,
            cwe: None,
            reference: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Injection,
    Auth,
    DataExposure,
    Misconfig,
    Regression,
    Fuzz,
    Recon,
    Unknown,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Injection => write!(f, "injection"),
            Category::Auth => write!(f, "auth"),
            Category::DataExposure => write!(f, "data_exposure"),
            Category::Misconfig => write!(f, "misconfiguration"),
            Category::Regression => write!(f, "regression"),
            Category::Fuzz => write!(f, "fuzz"),
            Category::Recon => write!(f, "recon"),
            Category::Unknown => write!(f, "unknown"),
        }
    }
}
