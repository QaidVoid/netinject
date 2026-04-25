use anyhow::Result;

use crate::cli::args::Cli;

pub async fn run(_cli: &Cli, pipeline: Option<&str>, _target_file: Option<&str>) -> Result<()> {
    let pipeline_name = pipeline.unwrap_or("full-api-scan");
    tracing::info!(pipeline = %pipeline_name, "executing pipeline");
    // TODO: load config, resolve pipeline, execute steps
    println!("Pipeline '{pipeline_name}' execution not yet implemented.");
    Ok(())
}
