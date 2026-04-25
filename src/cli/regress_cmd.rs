use anyhow::Result;

use crate::cli::args::Cli;

pub async fn run(_cli: &Cli, _against: Option<&str>, _paths: Option<&str>) -> Result<()> {
    tracing::info!("running regression check");
    // TODO: load baseline, capture current, compare
    println!("Regression check not yet implemented.");
    Ok(())
}
