use crate::adapters::AdapterRegistry;
use crate::config::AppConfig;
use crate::finding::Finding;

/// A pipeline is a sequence of adapter runs.
#[derive(Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug, Clone)]
pub struct PipelineStep {
    pub adapter: String,
    pub label: Option<String>,
    pub depends_on: Option<String>,
}

/// Execute a pipeline given a config definition.
pub async fn execute_pipeline(
    pipeline_name: &str,
    config: &AppConfig,
    registry: &AdapterRegistry,
    session_id: uuid::Uuid,
) -> anyhow::Result<Vec<Finding>> {
    let pipeline_def = config
        .pipeline
        .iter()
        .find(|p| p.name == pipeline_name)
        .ok_or_else(|| anyhow::anyhow!("pipeline '{pipeline_name}' not found in config"))?;

    let mut all_findings = Vec::new();
    let ctx = crate::adapters::RunContext {
        session_id,
        config: config.clone(),
        auth_headers: vec![],
        extra_args: std::collections::HashMap::new(),
    };

    for step in &pipeline_def.steps {
        let adapter = registry
            .get(&step.adapter)
            .ok_or_else(|| anyhow::anyhow!("adapter '{}' not found", step.adapter))?;

        tracing::info!(step = %step.adapter, label = ?step.label, "executing pipeline step");

        let input = crate::adapters::AdapterInput::Urls(
            config
                .project
                .target
                .as_deref()
                .map(|t| vec![t.to_string()])
                .unwrap_or_default(),
        );

        let findings = adapter.run(input, &ctx).await?;
        tracing::info!(step = %step.adapter, findings = findings.len(), "step completed");
        all_findings.extend(findings);
    }

    Ok(all_findings)
}
