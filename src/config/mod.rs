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
pub struct AppConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub scope: ScopeConfig,
    #[serde(default)]
    pub adapters: AdapterConfigs,
    #[serde(default)]
    pub auth: Vec<AuthProfile>,
    #[serde(default)]
    pub pipeline: Vec<PipelineConfig>,
    #[serde(default)]
    pub regression: RegressionConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub spec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "default_max_rate")]
    pub max_rate: u32,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            include: vec![],
            exclude: vec![],
            max_rate: default_max_rate(),
            max_concurrent: default_max_concurrent(),
        }
    }
}

fn default_max_rate() -> u32 {
    50
}
fn default_max_concurrent() -> u32 {
    10
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdapterConfigs {
    #[serde(default)]
    pub ffuf: FfufConfig,
    #[serde(default)]
    pub nuclei: NucleiConfig,
    #[serde(default)]
    pub httpx: HttpxConfig,
    #[serde(default)]
    pub sqlmap: SqlmapConfig,
    #[serde(default)]
    pub mitmproxy: MitmproxyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfufConfig {
    #[serde(default = "default_wordlist")]
    pub wordlist: String,
    #[serde(default = "default_threads")]
    pub threads: u32,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default)]
    pub recursive: bool,
}

impl Default for FfufConfig {
    fn default() -> Self {
        Self {
            wordlist: default_wordlist(),
            threads: default_threads(),
            timeout: default_timeout(),
            recursive: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleiConfig {
    #[serde(default)]
    pub templates: Vec<String>,
    #[serde(default)]
    pub severity: Vec<String>,
    #[serde(default = "default_nuclei_rate")]
    pub rate_limit: u32,
}

impl Default for NucleiConfig {
    fn default() -> Self {
        Self {
            templates: vec![],
            severity: vec![],
            rate_limit: default_nuclei_rate(),
        }
    }
}

fn default_nuclei_rate() -> u32 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpxConfig {
    #[serde(default = "default_threads")]
    pub threads: u32,
    #[serde(default = "default_nuclei_rate")]
    pub rate_limit: u32,
    #[serde(default)]
    pub tech_detect: bool,
}

impl Default for HttpxConfig {
    fn default() -> Self {
        Self {
            threads: default_threads(),
            rate_limit: 150,
            tech_detect: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlmapConfig {
    #[serde(default = "default_sqlmap_level")]
    pub level: u32,
    #[serde(default = "default_sqlmap_risk")]
    pub risk: u32,
    #[serde(default = "default_bool_true")]
    pub batch: bool,
}

impl Default for SqlmapConfig {
    fn default() -> Self {
        Self {
            level: default_sqlmap_level(),
            risk: default_sqlmap_risk(),
            batch: true,
        }
    }
}

fn default_sqlmap_level() -> u32 {
    3
}
fn default_sqlmap_risk() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitmproxyConfig {
    #[serde(default = "default_listen_host")]
    pub listen_host: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default)]
    pub upstream_proxy: String,
}

impl Default for MitmproxyConfig {
    fn default() -> Self {
        Self {
            listen_host: default_listen_host(),
            listen_port: default_listen_port(),
            upstream_proxy: String::new(),
        }
    }
}

fn default_listen_host() -> String {
    "127.0.0.1".into()
}
fn default_listen_port() -> u16 {
    8080
}

fn default_wordlist() -> String {
    "/usr/share/seclists/Discovery/Web-Content/common.txt".into()
}
fn default_threads() -> u32 {
    40
}
fn default_timeout() -> u32 {
    10
}
fn default_bool_true() -> bool {
    true
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegressionConfig {
    #[serde(default = "default_regression_status")]
    pub status_code_change: String,
    #[serde(default = "default_regression_schema")]
    pub schema_drift: String,
    #[serde(default = "default_regression_timing")]
    pub timing_threshold: f64,
    #[serde(default = "default_regression_header")]
    pub header_change: String,
    #[serde(default = "default_regression_body")]
    pub body_hash_change: String,
}

fn default_regression_status() -> String {
    "breaking".into()
}
fn default_regression_schema() -> String {
    "breaking".into()
}
fn default_regression_timing() -> f64 {
    2.0
}
fn default_regression_header() -> String {
    "info".into()
}
fn default_regression_body() -> String {
    "warning".into()
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
