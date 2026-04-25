use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = netinject::cli::args::Cli::parse();

    // Respect --no-color and NO_COLOR environment variable
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        console::set_colors_enabled(false);
        console::set_colors_enabled_stderr(false);
    }

    // Initialize tracing
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("netinject=debug")
            .init();
    } else if !cli.quiet {
        tracing_subscriber::fmt()
            .with_env_filter("netinject=info")
            .init();
    }

    // Use tokio runtime for all commands
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main(&cli))
}

async fn async_main(cli: &netinject::cli::args::Cli) -> anyhow::Result<()> {
    use netinject::cli::args::Commands;

    match &cli.command {
        Commands::Run {
            pipeline,
            target_file,
        } => netinject::cli::run_cmd::run(cli, pipeline.as_deref(), target_file.as_deref()).await,
        Commands::Recon => netinject::cli::recon_cmd::run(cli).await,
        Commands::Fuzz { fuzz_point } => {
            netinject::cli::fuzz_cmd::run(cli, fuzz_point.as_deref()).await
        }
        Commands::Scan => netinject::cli::scan_cmd::run(cli).await,
        Commands::Baseline { subcommand } => {
            netinject::cli::baseline_cmd::run(cli, subcommand).await
        }
        Commands::Regress { against, paths } => {
            netinject::cli::regress_cmd::run(cli, against.as_deref(), paths.as_deref()).await
        }
        Commands::Sessions { subcommand } => {
            netinject::cli::sessions_cmd::run(cli, subcommand).await
        }
        Commands::Replay { session_id, modify } => {
            netinject::cli::replay_cmd::run(cli, session_id, modify.as_ref()).await
        }
        Commands::Init { name } => Ok(netinject::cli::init_cmd::run(cli, name.as_deref())?),
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            use clap_complete::generate;
            let mut cmd = netinject::cli::args::Cli::command();
            let name = cmd.get_name().to_string();
            generate(*shell, &mut cmd, name, &mut std::io::stdout());
            Ok(())
        }
        Commands::Check => Ok(netinject::cli::check_cmd::run(cli)?),
    }
}
