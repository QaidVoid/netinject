use anyhow::Result;

use crate::cli::args::Cli;

pub async fn run(_cli: &Cli) -> Result<()> {
    tracing::info!("running scan (nuclei)");
    // TODO: invoke nuclei adapter
    println!("Scan not yet implemented.");
    Ok(())
}
