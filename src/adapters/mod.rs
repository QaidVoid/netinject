pub mod ffuf;
pub mod httpx;
pub mod mitmproxy;
pub mod nuclei;
pub mod sqlmap;

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::finding::Finding;

/// Context provided to every adapter run.
pub struct RunContext {
    pub session_id: uuid::Uuid,
    pub config: crate::config::AppConfig,
    pub auth_headers: Vec<(String, String)>,
    pub extra_args: HashMap<String, String>,
}

/// Input to an adapter — either a list of URLs or a spec-driven list of endpoints.
pub enum AdapterInput {
    Urls(Vec<String>),
    Endpoints(Vec<crate::spec::Endpoint>),
}

impl AdapterInput {
    pub fn urls(&self) -> Vec<String> {
        match self {
            AdapterInput::Urls(urls) => urls.clone(),
            AdapterInput::Endpoints(endpoints) => endpoints
                .iter()
                .map(|e| format!("{} {}", e.method, e.path))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            AdapterInput::Urls(urls) => urls.is_empty(),
            AdapterInput::Endpoints(endpoints) => endpoints.is_empty(),
        }
    }
}

/// Core adapter trait — every tool implements this.
#[async_trait]
pub trait Adapter: Send + Sync {
    /// Adapter name (e.g., "ffuf", "nuclei").
    fn name(&self) -> &str;

    /// Check if the underlying tool binary is available.
    fn check_available(&self) -> Result<bool>;

    /// Get the tool's version string.
    fn version(&self) -> Result<String>;

    /// Execute the tool and return normalized findings.
    async fn run(&self, input: AdapterInput, ctx: &RunContext) -> Result<Vec<Finding>>;
}

/// Registry of all available adapters.
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn Adapter>>,
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            adapters: HashMap::new(),
        };
        reg.register(Box::new(ffuf::FfufAdapter));
        reg.register(Box::new(nuclei::NucleiAdapter));
        reg.register(Box::new(httpx::HttpxAdapter));
        reg.register(Box::new(sqlmap::SqlmapAdapter));
        reg.register(Box::new(mitmproxy::MitmproxyAdapter));
        reg
    }

    fn register(&mut self, adapter: Box<dyn Adapter>) {
        let name = adapter.name().to_string();
        self.adapters.insert(name, adapter);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Adapter> {
        self.adapters.get(name).map(|b| b.as_ref())
    }

    pub fn all(&self) -> Vec<&dyn Adapter> {
        self.adapters.values().map(|b| b.as_ref()).collect()
    }

    pub fn names(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}
