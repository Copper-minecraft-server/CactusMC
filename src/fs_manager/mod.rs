use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
mod utils;
use log::{error, warn};

use crate::{consts, gracefully_exit};

// Initializes the server's required files and directories
pub fn init() -> std::io::Result<()> {
    create_server_properties()?;

    eula()
}

/// Checks if the eula is agreed, if not creates it.
fn eula() -> io::Result<()> {
    let path = Path::new(consts::filepaths::EULA);
    if !path.exists() {
        create_eula()?;
        warn!("Please agree to the 'eula.txt' and start the server again.");
        gracefully_exit(0);
    } else {
        let is_agreed_eula = check_eula()?;
        if !is_agreed_eula {
            error!("Cannot start the server, please agree to the 'eula.txt'");
            gracefully_exit(-1);
        }
        Ok(())
    }
}

/// Creates the 'server.properties' file if it does not already exist.
fn create_server_properties() -> io::Result<()> {
    let path = Path::new(consts::filepaths::PROPERTIES);
    let content = consts::file_content::server_properties();

    utils::create_file(&path, &content)
}

/// Creates the 'eula.txt' file if it does not already exist.
fn create_eula() -> io::Result<()> {
    let path = Path::new(consts::filepaths::EULA);
    let content = consts::file_content::eula();

    utils::create_file(&path, &content)
}

/// Check if the 'eula.txt' has been agreed to.
fn check_eula() -> io::Result<bool> {
    let file = File::open(Path::new(consts::filepaths::EULA))?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("eula=") {
            let eula_value = line.split('=').nth(1).unwrap_or("").to_lowercase();
            return Ok(eula_value == "true");
        }
    }

    Ok(false)
}