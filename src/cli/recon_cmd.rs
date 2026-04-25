use anyhow::Result;

use crate::cli::args::Cli;

pub async fn run(_cli: &Cli) -> Result<()> {
    tracing::info!("running recon (httpx)");
    // TODO: invoke httpx adapter
    println!("Recon not yet implemented.");
    Ok(())
}
