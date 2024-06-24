pub mod backup;
pub mod config;
pub mod log;
pub mod summary;

use std::process;
use clap::Parser;
use crate::{error::*, cli::*, config::*, log::*, run};

pub fn run_main() -> process::ExitCode {
    let cli = Cli::parse();
    if let Ok(config) = read_cli_config(&cli) {
        Log::init(Some(&config), Some(&cli));
        match run_with_config(cli, config) {
            Ok(true) => process::ExitCode::SUCCESS,
            Ok(false) => process::ExitCode::FAILURE,
            Err(e) => {
                Log::get().error(&e.to_string());
                process::ExitCode::FAILURE
            }
        }
    } else {
        Log::init(None, Some(&cli));
        match run_with(cli) {
            Ok(true) => process::ExitCode::SUCCESS,
            Ok(false) => process::ExitCode::FAILURE,
            Err(e) => {
                Log::get().error(&e.to_string());
                process::ExitCode::FAILURE
            }
        }
    }
}

pub fn run_with(cli: Cli) -> Result<bool> {
    match &cli.subcommand {
        Command::Backup(subcmd) => run::backup::run_backup(&cli, subcmd, None).map(|_| Ok(true))?,
        Command::Config(subcmd) => run::config::run_config(&cli, subcmd),
        Command::Log(subcmd) => run::log::run_log(&cli, subcmd),
        Command::Summary => run::summary::run_summary(&cli),
    }
}

pub fn run_with_config(cli: Cli, config: BackupConfig) -> Result<bool> {
    match &cli.subcommand {
        Command::Backup(subcmd) => run::backup::run_backup(&cli, subcmd, Some(&config)).map(|_| Ok(true))?,
        Command::Config(subcmd) => run::config::run_config(&cli, subcmd),
        Command::Log(subcmd) => run::log::run_log(&cli, subcmd),
        Command::Summary => run::summary::run_summary(&cli),
    }
}


