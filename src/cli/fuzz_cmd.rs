use anyhow::Result;

use crate::cli::args::Cli;

pub async fn run(_cli: &Cli, _fuzz_point: Option<&str>) -> Result<()> {
    tracing::info!("running fuzz (ffuf)");
    // TODO: invoke ffuf adapter
    println!("Fuzz not yet implemented.");
    Ok(())
}
