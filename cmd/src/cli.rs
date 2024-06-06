use std::path::PathBuf;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[clap(short, help = "Configuration file [non-default]", value_parser = validate_config_file)]
    pub config_file: Option<PathBuf>,

    #[command(subcommand)]
    pub subcommand: Command,

}


#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(name = "backup", about = "Performs all scheduled backups as configured")]
    Backup,
    #[command(name = "config", about = "Initializes or validates the user's bak9 configuration")]
    Config 
}

fn validate_config_file(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path)
        .canonicalize()
        .map_err(|_| format!("Config file not found: {}", path.cyan()))?;

    if !path.is_file() {
        Err(format!("Config path is not a file: {}", path.to_str().unwrap().cyan()))
    } else {
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_cli() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert()
    }
}
