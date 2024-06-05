pub mod error;
pub mod config;
pub mod schedule;
pub mod paths;
pub mod cli;
pub mod cmd;
pub mod backup;
pub mod testing;

use chrono::Timelike;
use colored::Colorize;
use crate::error::Result;

pub const BAK9: &'static str = "bak9";

pub fn make_log_prefix(topic: &'static str, color: colored::Color) -> String {
    let now = chrono::Local::now();
    let prefix = format!("[{:0<2}:{:0<2}:{:0<2} {topic}]:",
        now.hour(),
        now.minute(),
        now.second());

    prefix.color(color).to_string()
}

pub fn bak9_error_log_prefix() -> String {
    make_log_prefix(BAK9, colored::Color::Red)
}

pub fn bak9_info_log_prefix() -> String {
    make_log_prefix(BAK9, colored::Color::Green)
}

pub fn run() -> std::process::ExitCode {
    let config = match config::read_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {e}", bak9_error_log_prefix());
            return std::process::ExitCode::FAILURE;
        }
    };

    match run_with(config) {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e}", bak9_error_log_prefix());
            std::process::ExitCode::FAILURE
        }
    }
}

pub fn run_with(config: config::BackupConfig) -> Result<()> {
    verify_environment(&config)?;
    Ok(())
}

/// Verify that the runtime environment that has been configured is valid.  
/// Verification:
/// - Directories need to exist
pub fn verify_environment(config: &config::BackupConfig) -> Result<()> {
    paths::verify_backup_dirs(config)?;

    Ok(())
}

