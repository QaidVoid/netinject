use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::auth::AuthProfile;
use crate::scope::ScopeChecker;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to create scope checker: {0}")]
    Scope(#[from] crate::scope::ScopeError),
}

/// Resolved application configuration, merged from all sources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub project: ProjectConfig,
    pub scope: ScopeConfig,
    pub adapters: AdapterConfigs,
    pub auth: Vec<AuthProfile>,
    pub pipeline: Vec<PipelineConfig>,
    pub regression: RegressionConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub target: Option<String>,
    pub spec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScopeConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub max_rate: u32,
    pub max_concurrent: u32,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            include: vec![],
            exclude: vec![],
            max_rate: 50,
            max_concurrent: 10,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AdapterConfigs {
    pub ffuf: FfufConfig,
    pub nuclei: NucleiConfig,
    pub httpx: HttpxConfig,
    pub sqlmap: SqlmapConfig,
    pub mitmproxy: MitmproxyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FfufConfig {
    pub wordlist: String,
    pub threads: u32,
    pub timeout: u32,
    pub recursive: bool,
}

impl Default for FfufConfig {
    fn default() -> Self {
        Self {
            wordlist: "/usr/share/seclists/Discovery/Web-Content/common.txt".into(),
            threads: 40,
            timeout: 10,
            recursive: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NucleiConfig {
    pub templates: Vec<String>,
    pub severity: Vec<String>,
    pub rate_limit: u32,
}

impl Default for NucleiConfig {
    fn default() -> Self {
        Self {
            templates: vec![],
            severity: vec![],
            rate_limit: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpxConfig {
    pub threads: u32,
    pub rate_limit: u32,
    pub tech_detect: bool,
}

impl Default for HttpxConfig {
    fn default() -> Self {
        Self {
            threads: 40,
            rate_limit: 150,
            tech_detect: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SqlmapConfig {
    pub level: u32,
    pub risk: u32,
    pub batch: bool,
}

impl Default for SqlmapConfig {
    fn default() -> Self {
        Self {
            level: 3,
            risk: 2,
            batch: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MitmproxyConfig {
    pub listen_host: String,
    pub listen_port: u16,
    pub upstream_proxy: String,
}

impl Default for MitmproxyConfig {
    fn default() -> Self {
        Self {
            listen_host: "127.0.0.1".into(),
            listen_port: 8080,
            upstream_proxy: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub adapter: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub depends_on: Option<String>,
    #[serde(default)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RegressionConfig {
    pub status_code_change: String,
    pub schema_drift: String,
    pub timing_threshold: f64,
    pub header_change: String,
    pub body_hash_change: String,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            status_code_change: "breaking".into(),
            schema_drift: "breaking".into(),
            timing_threshold: 2.0,
            header_change: "info".into(),
            body_hash_change: "warning".into(),
        }
    }
}

/// Load config from a TOML file.
pub fn load_config(path: &Path) -> Result<AppConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Resolve config with precedence: CLI flags > project config > user config > defaults.
pub fn resolve_config(
    cli_target: Option<&str>,
    cli_spec: Option<&str>,
    project_config: Option<&Path>,
) -> Result<AppConfig, ConfigError> {
    let mut config = if let Some(path) = project_config {
        load_config(path)?
    } else {
        AppConfig::default()
    };

    // CLI overrides
    if let Some(target) = cli_target {
        config.project.target = Some(target.to_string());
    }
    if let Some(spec) = cli_spec {
        config.project.spec = Some(spec.to_string());
    }

    // Resolve auth env vars
    for auth in &mut config.auth {
        auth.resolve_env_vars();
    }

    Ok(config)
}

/// Build a scope checker from config.
pub fn build_scope_checker(config: &AppConfig) -> Result<ScopeChecker, ConfigError> {
    if config.scope.include.is_empty() {
        Ok(ScopeChecker::allow_all())
    } else {
        ScopeChecker::new(&config.scope.include, &config.scope.exclude).map_err(ConfigError::from)
    }
}
