use clap::{Parser, Subcommand};

use crate::report::OutputFormat;

#[derive(Parser)]
#[command(name = "netinject")]
#[command(about = "Lightweight API security testing orchestrator")]
#[command(version)]
pub struct Cli {
    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Override target URL
    #[arg(long, global = true)]
    pub target: Option<String>,

    /// Path to OpenAPI/Swagger spec
    #[arg(long, global = true)]
    pub spec: Option<String>,

    /// Auth profile to use
    #[arg(long, global = true)]
    pub auth: Option<String>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// Output file path
    #[arg(long, global = true)]
    pub output: Option<String>,

    /// Verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Only show findings, suppress progress
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Show what would be run without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a full pipeline (recon → fuzz → scan → regress)
    Run {
        /// Pipeline name from config
        #[arg(long)]
        pipeline: Option<String>,

        /// File with target URLs (one per line)
        #[arg(long)]
        target_file: Option<String>,
    },

    /// Run discovery/probing only (httpx)
    Recon,

    /// Run fuzzing only (ffuf)
    Fuzz {
        /// Fuzz point (e.g., parameter name, directory)
        #[arg(long)]
        fuzz_point: Option<String>,
    },

    /// Run vulnerability scanning only (nuclei)
    Scan,

    /// Baseline capture and regression detection
    Baseline {
        #[command(subcommand)]
        subcommand: BaselineCommands,
    },

    /// Check current state against a baseline
    Regress {
        /// Baseline to check against
        #[arg(long)]
        against: Option<String>,

        /// Only check specific endpoint paths (comma-separated)
        #[arg(long)]
        paths: Option<String>,
    },

    /// Browse and compare past sessions
    Sessions {
        #[command(subcommand)]
        subcommand: SessionCommands,
    },

    /// Replay captured requests with modifications
    Replay {
        /// Session ID to replay from
        session_id: String,

        /// Modify request before replay (header:value pairs)
        #[arg(long)]
        modify: Option<Vec<String>>,
    },

    /// Initialize a new project with config template
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
    },

    /// Verify all required tools are installed
    Check,
}

#[derive(Subcommand)]
pub enum BaselineCommands {
    /// Capture a baseline from the live API
    Capture,

    /// List captured baselines
    List,

    /// Diff two baselines
    Diff {
        /// First baseline
        baseline_a: String,
        /// Second baseline
        baseline_b: String,
    },
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List all sessions
    List,

    /// Show findings from a specific session
    Show {
        /// Session ID
        id: String,
    },

    /// Compare findings between sessions
    Diff {
        /// First session ID
        id_a: String,
        /// Second session ID
        id_b: String,
    },

    /// Export session to a file
    Export {
        /// Session ID
        id: String,
    },
}
