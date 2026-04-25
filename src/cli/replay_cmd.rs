use anyhow::Result;

use crate::cli::args::Cli;

pub async fn run(_cli: &Cli, _session_id: &str, _modify: Option<&Vec<String>>) -> Result<()> {
    tracing::info!("replaying requests");
    println!("Replay not yet implemented.");
    Ok(())
}
