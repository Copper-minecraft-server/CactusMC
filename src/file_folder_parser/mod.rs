use std::fs::File;
use std::io::{self, Write ,BufRead};
use std::path::Path;
use colored::Colorize;


pub fn create_server_properties(content:&str,file_path:&'static str,formatted_time:&str) -> io::Result<()> {
    let path = Path::new(file_path);
    let final_input = format!(
        "#Minecraft server properties\n#{}\n{}",
        formatted_time,
        content,
    );
    // verify if the file already exist
    if path.exists() {
        println!("the file \"{}\" already exists, the program will use this one.",file_path.blue());
    } else {
        let mut file = File::create(path)?;
        file.write_all(final_input.as_bytes())?;
        println!("The file \"{}\" has been created.",file_path.blue())
    }
    Ok(())
}//test passed

pub fn create_eula(file_path:&'static str,formatted_time:&str) -> io::Result<()> {


    let final_input = format!(
        "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n#{}\neula=false",
        formatted_time,

    );

    let path = Path::new(file_path);
    if path.exists() {
        println!("the file {} is already been created",file_path.green());
    } else {
        let mut file = File::create(path)?;
        file.write_all(final_input.as_bytes())?;
        println!("Creation of the file {}",file_path.red())
    }

    Ok(())
}


pub fn check_eula(path: &'static str) -> bool {
    println!("Reading the file {}…",path);
    if let Ok(file) = File::open(Path::new(path)) {
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("eula=") {
                    let eula_value = line.split('=').nth(1).unwrap_or("").to_lowercase();
                    return eula_value == "true";
                }
            }
        }
    }

    false
}



