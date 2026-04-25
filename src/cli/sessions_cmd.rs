use anyhow::Result;

use crate::cli::args::{Cli, SessionCommands};

pub async fn run(_cli: &Cli, subcommand: &SessionCommands) -> Result<()> {
    match subcommand {
        SessionCommands::List => {
            let store = open_default_store()?;
            let sessions = store.list_sessions()?;
            if sessions.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }

            let rows: Vec<(String, String, String, String)> = sessions
                .iter()
                .map(|s| {
                    (
                        s.id.to_string()[..8].to_string(),
                        s.created_at.format("%Y-%m-%d %H:%M").to_string(),
                        s.target.clone(),
                        s.status.to_string(),
                    )
                })
                .collect();

            let mut table = tabled::Table::new(rows);
            table.with(tabled::settings::Style::modern());
            println!("{table}");
        }
        SessionCommands::Show { id } => {
            tracing::info!(session_id = %id, "showing session");
            println!("Session show not yet implemented.");
        }
        SessionCommands::Diff { id_a, id_b } => {
            tracing::info!("diffing sessions: {id_a} vs {id_b}");
            println!("Session diff not yet implemented.");
        }
        SessionCommands::Export { id } => {
            tracing::info!(session_id = %id, "exporting session");
            println!("Session export not yet implemented.");
        }
    }
    Ok(())
}

fn open_default_store() -> Result<crate::session::store::SessionStore> {
    let base_dir = dirs_home()?;
    let db_path = base_dir.join("store.db");
    crate::session::store::SessionStore::open(&db_path)
}

fn dirs_home() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("cannot determine home directory"))?;
    Ok(std::path::PathBuf::from(home).join(".netinject"))
}
