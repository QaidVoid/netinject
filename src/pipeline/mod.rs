use std::collections::HashMap;

use anyhow::Result;
use thiserror::Error;

use crate::adapters::{AdapterInput, AdapterRegistry, RunContext};
use crate::config::{AppConfig, PipelineConfig};
use crate::finding::Finding;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("pipeline '{name}' not found in config")]
    NotFound { name: String },

    #[error("adapter '{name}' not found")]
    AdapterNotFound { name: String },

    #[error("cycle detected in pipeline step dependencies")]
    CyclicDependency,

    #[error("step '{step}' depends on '{dep}', but '{dep}' was not found")]
    MissingDependency { step: String, dep: String },
}

/// Resolved execution order for pipeline steps.
#[derive(Debug)]
struct ExecutionPlan {
    /// Steps in topological order (dependencies first).
    steps: Vec<usize>,
}

impl ExecutionPlan {
    /// Build an execution plan from pipeline config by topologically sorting steps
    /// based on their `depends_on` fields.
    fn build(pipeline: &PipelineConfig) -> Result<Self, PipelineError> {
        let n = pipeline.steps.len();
        let label_to_idx: HashMap<String, usize> = pipeline
            .steps
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.label.as_ref().map(|l| (l.clone(), i)))
            .collect();

        // Build adjacency list: edges[i] = set of steps that depend on step i
        let mut in_degree = vec![0u32; n];
        let mut edges: Vec<Vec<usize>> = vec![vec![]; n];

        for (i, step) in pipeline.steps.iter().enumerate() {
            if let Some(ref dep_label) = step.depends_on {
                let &dep_idx =
                    label_to_idx
                        .get(dep_label)
                        .ok_or(PipelineError::MissingDependency {
                            step: step.label.clone().unwrap_or(step.adapter.clone()),
                            dep: dep_label.clone(),
                        })?;
                edges[dep_idx].push(i);
                in_degree[i] += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut sorted = Vec::with_capacity(n);

        while let Some(idx) = queue.pop() {
            sorted.push(idx);
            for &dep in &edges[idx] {
                in_degree[dep] -= 1;
                if in_degree[dep] == 0 {
                    queue.push(dep);
                }
            }
        }

        if sorted.len() != n {
            return Err(PipelineError::CyclicDependency);
        }

        Ok(Self { steps: sorted })
    }
}

/// Extract live URLs from findings produced by a step.
/// These URLs become input to downstream steps.
fn urls_from_findings(findings: &[Finding]) -> Vec<String> {
    let mut urls: Vec<String> = findings
        .iter()
        .map(|f| f.url.clone())
        .filter(|u| !u.is_empty())
        .collect();
    urls.sort();
    urls.dedup();
    urls
}

/// Execute a pipeline by name from config.
pub async fn execute_pipeline(
    pipeline_name: &str,
    config: &AppConfig,
    registry: &AdapterRegistry,
    ctx: &RunContext,
    initial_urls: Vec<String>,
    dry_run: bool,
) -> Result<Vec<Finding>> {
    let pipeline = config
        .pipeline
        .iter()
        .find(|p| p.name == pipeline_name)
        .ok_or_else(|| PipelineError::NotFound {
            name: pipeline_name.to_string(),
        })?;

    let plan = ExecutionPlan::build(pipeline)?;

    if dry_run {
        return dry_run_plan(pipeline, &plan, &initial_urls);
    }

    let mut all_findings = Vec::new();
    // Track URLs produced by each step label for dependency resolution
    let mut step_outputs: HashMap<String, Vec<String>> = HashMap::new();

    for &step_idx in &plan.steps {
        let step = &pipeline.steps[step_idx];
        let label = step
            .label
            .clone()
            .unwrap_or_else(|| format!("step-{}", step_idx));

        // Resolve input URLs for this step
        let input_urls = if let Some(ref dep_label) = step.depends_on {
            step_outputs.get(dep_label).cloned().unwrap_or_default()
        } else {
            initial_urls.clone()
        };

        if input_urls.is_empty() {
            tracing::warn!(step = %label, "skipping step — no input URLs");
            continue;
        }

        let adapter =
            registry
                .get(&step.adapter)
                .ok_or_else(|| PipelineError::AdapterNotFound {
                    name: step.adapter.clone(),
                })?;

        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_style(
            indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!(
            "{label} ({adapter})",
            label = label,
            adapter = step.adapter
        ));

        tracing::info!(
            step = %label,
            adapter = %step.adapter,
            input_count = input_urls.len(),
            "executing pipeline step"
        );

        let input = AdapterInput::Urls(input_urls);
        let findings = adapter.run(input, ctx).await?;

        pb.finish_with_message(format!(
            "{label} ({adapter}) — {count} findings",
            label = label,
            adapter = step.adapter,
            count = findings.len()
        ));

        tracing::info!(
            step = %label,
            findings = findings.len(),
            "step completed"
        );

        // Store output URLs for dependent steps
        step_outputs.insert(label, urls_from_findings(&findings));
        all_findings.extend(findings);
    }

    Ok(all_findings)
}

/// Show what would be executed without running.
fn dry_run_plan(
    pipeline: &PipelineConfig,
    plan: &ExecutionPlan,
    initial_urls: &[String],
) -> Result<Vec<Finding>> {
    println!("Pipeline: {}\n", pipeline.name);
    if !pipeline.description.is_empty() {
        println!("  {}\n", pipeline.description);
    }

    for &step_idx in &plan.steps {
        let step = &pipeline.steps[step_idx];
        let label = step
            .label
            .clone()
            .unwrap_or_else(|| format!("step-{}", step_idx));
        let dep = step
            .depends_on
            .as_deref()
            .map(|d| format!(" (depends on: {d})"))
            .unwrap_or_default();

        println!("  → [{label}] {adapter}{dep}", adapter = step.adapter);
    }

    println!(
        "\n  Initial targets: {}",
        if initial_urls.is_empty() {
            "none (set --target or --target-file)".to_string()
        } else {
            initial_urls.join(", ")
        }
    );
    println!("  (dry run — no tools executed)");

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PipelineConfig, PipelineStep};

    fn make_pipeline(steps: Vec<PipelineStep>) -> PipelineConfig {
        PipelineConfig {
            name: "test-pipeline".into(),
            description: String::new(),
            steps,
        }
    }

    #[test]
    fn test_execution_plan_sequential() {
        let pipeline = make_pipeline(vec![
            PipelineStep {
                adapter: "httpx".into(),
                label: Some("recon".into()),
                depends_on: None,
                config: None,
            },
            PipelineStep {
                adapter: "nuclei".into(),
                label: Some("scan".into()),
                depends_on: Some("recon".into()),
                config: None,
            },
        ]);

        let plan = ExecutionPlan::build(&pipeline).unwrap();
        assert_eq!(plan.steps, vec![0, 1]);
    }

    #[test]
    fn test_execution_plan_parallel() {
        // Two steps with no dependencies — order doesn't matter
        let pipeline = make_pipeline(vec![
            PipelineStep {
                adapter: "nuclei".into(),
                label: Some("scan".into()),
                depends_on: None,
                config: None,
            },
            PipelineStep {
                adapter: "httpx".into(),
                label: Some("recon".into()),
                depends_on: None,
                config: None,
            },
        ]);

        let plan = ExecutionPlan::build(&pipeline).unwrap();
        assert_eq!(plan.steps.len(), 2);
        // Both have in-degree 0, so they should both appear
        assert!(plan.steps.contains(&0));
        assert!(plan.steps.contains(&1));
    }

    #[test]
    fn test_execution_plan_diamond_dependency() {
        // recon → fuzz, recon → scan (both depend on recon)
        let pipeline = make_pipeline(vec![
            PipelineStep {
                adapter: "httpx".into(),
                label: Some("recon".into()),
                depends_on: None,
                config: None,
            },
            PipelineStep {
                adapter: "ffuf".into(),
                label: Some("fuzz".into()),
                depends_on: Some("recon".into()),
                config: None,
            },
            PipelineStep {
                adapter: "nuclei".into(),
                label: Some("scan".into()),
                depends_on: Some("recon".into()),
                config: None,
            },
        ]);

        let plan = ExecutionPlan::build(&pipeline).unwrap();
        // recon must come first
        assert_eq!(plan.steps[0], 0);
        assert!(plan.steps.contains(&1));
        assert!(plan.steps.contains(&2));
    }

    #[test]
    fn test_execution_plan_missing_dependency() {
        let pipeline = make_pipeline(vec![PipelineStep {
            adapter: "nuclei".into(),
            label: Some("scan".into()),
            depends_on: Some("nonexistent".into()),
            config: None,
        }]);

        let result = ExecutionPlan::build(&pipeline);
        assert!(result.is_err());
        match result.unwrap_err() {
            PipelineError::MissingDependency { step, dep } => {
                assert_eq!(step, "scan");
                assert_eq!(dep, "nonexistent");
            }
            e => panic!("wrong error: {e}"),
        }
    }

    #[test]
    fn test_execution_plan_cycle_detected() {
        let pipeline = make_pipeline(vec![
            PipelineStep {
                adapter: "httpx".into(),
                label: Some("a".into()),
                depends_on: Some("b".into()),
                config: None,
            },
            PipelineStep {
                adapter: "nuclei".into(),
                label: Some("b".into()),
                depends_on: Some("a".into()),
                config: None,
            },
        ]);

        let result = ExecutionPlan::build(&pipeline);
        assert!(matches!(
            result.unwrap_err(),
            PipelineError::CyclicDependency
        ));
    }

    #[test]
    fn test_urls_from_findings() {
        let session_id = uuid::Uuid::new_v4();
        let mut f1 = Finding::new(session_id, "httpx");
        f1.url = "https://example.com".into();
        let mut f2 = Finding::new(session_id, "httpx");
        f2.url = "https://other.com".into();
        let mut f3 = Finding::new(session_id, "httpx");
        f3.url = "https://example.com".into(); // duplicate

        let urls = urls_from_findings(&[f1, f2, f3]);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.contains(&"https://other.com".to_string()));
    }
}
