use crate::fs_manager;
use clap::Parser;
use log::error;

#[derive(Parser)]
#[command(name = "CactusMC")]
#[command(about = "This is the about, please change", long_about = None)]
struct Cli {
    /// Removes all server-related files except the server executable.
    #[arg(short, long)]
    remove_files: bool,
}

/// Retrieves args and initializes the argument parsing logic.
pub fn init() {
    let args = Cli::parse();

    if args.remove_files {
        if let Err(e) = fs_manager::clean_files() {
            error!("Error(s) when cleaning files: {e}");
        }
    }
}

