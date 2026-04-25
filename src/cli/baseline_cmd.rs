use anyhow::Result;

use crate::cli::args::{BaselineCommands, Cli};

pub async fn run(_cli: &Cli, subcommand: &BaselineCommands) -> Result<()> {
    match subcommand {
        BaselineCommands::Capture => {
            tracing::info!("capturing baseline");
            println!("Baseline capture not yet implemented.");
        }
        BaselineCommands::List => {
            tracing::info!("listing baselines");
            println!("Baseline list not yet implemented.");
        }
        BaselineCommands::Diff {
            baseline_a,
            baseline_b,
        } => {
            tracing::info!("diffing baselines: {baseline_a} vs {baseline_b}");
            println!("Baseline diff not yet implemented.");
        }
    }
    Ok(())
}
