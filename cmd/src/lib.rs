pub mod error;
pub mod config;
pub mod schedule;
pub mod paths;
pub mod cli;
pub mod cmd;
pub mod backup;
pub mod testing;

use std::io::Write;

use chrono::Timelike;
use clap::Parser;
use colored::Colorize;
use error::Error;
use crate::error::Result;

pub const BAK9: &'static str = "bak9";

pub fn make_log_prefix(topic: &str, status: &str, color: colored::Color) -> String {
    let now = chrono::Local::now();
    let prefix = format!("[{:0<2}:{:0<2}:{:0<2} {topic}]{status}",
        now.hour(),
        now.minute(),
        now.second());

    prefix.color(color).to_string()
}

pub fn bak9_error_log_prefix() -> String {
    make_log_prefix(BAK9, " error:", colored::Color::Red)
}

pub fn bak9_info_log_prefix() -> String {
    make_log_prefix(BAK9, "", colored::Color::Green)
}

pub fn run() -> std::process::ExitCode {
    let cli = cli::Cli::parse();
    match run_with(cli) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{} {e}", bak9_error_log_prefix());
            std::process::ExitCode::FAILURE
        }
    }
}

pub fn run_with(cli: cli::Cli) -> Result<std::process::ExitCode> {
    match cli.subcommand {
        cli::Command::Backup => run_backup(&cli),
        cli::Command::Config => run_config(&cli)
    }
}

fn run_config(cli: &cli::Cli) -> Result<std::process::ExitCode> {
    let config_path = config::select_config_path(cli)?;
    if !config_path.exists() {
        println!("{} Config file {} not found.", "warning:".yellow(), config_path.to_str().unwrap().cyan());
        print!("{} Would you like to create it now? {} ", "confirm:".bright_yellow(), "[y/N]:".magenta());

        std::io::stdout().flush()
            .expect("Failed to flush stdout");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .expect("Failed to read input");

        if input.trim().to_lowercase() == "y" {
            let config_dir = config_path.parent()
                .expect("Failed to get parent directory");
            std::fs::create_dir_all(config_dir)
                .map_err(|e| Error::new_file_io(config_dir, e))?;
            std::fs::write(&config_path, config::CONFIG_DEFAULTS)
                .map_err(|e| Error::new_file_io(&config_path, e))?;

            println!("Config file {} created from template.", config_path.to_str().unwrap().cyan());
            println!("Edit your config and run {} again to validate it.", "bak9 config".yellow());
            Ok(std::process::ExitCode::SUCCESS)
        } else {
            Ok(std::process::ExitCode::FAILURE)
        }
    } else {
        println!("{} Config file found at {}", "valid:".green(), config_path.to_str().unwrap());
        Ok(std::process::ExitCode::SUCCESS)
    }
}

fn run_backup(cli: &cli::Cli) -> Result<std::process::ExitCode> {
    let config = config::read_config(cli.config_file.as_ref())?;
    verify_environment(&config)?;
    Ok(std::process::ExitCode::SUCCESS)
}

/// Verify that the runtime environment that has been configured is valid.  
/// Verification:
/// - Directories need to exist
pub fn verify_environment(config: &config::BackupConfig) -> Result<()> {
    paths::verify_backup_dirs(config)?;

    Ok(())
}

