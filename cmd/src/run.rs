pub mod backup;
pub mod config;
pub mod log;
pub mod summary;

use std::process;
use clap::Parser;
use crate::{error::{self, Result}, cli, run};

pub fn run_default() -> process::ExitCode {
    let cli = cli::Cli::parse();
    match run_with(cli) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{} {e}", error::bak9_error_log_prefix());
            std::process::ExitCode::FAILURE
        }
    }
}

pub fn run_with(cli: cli::Cli) -> Result<process::ExitCode> {
    match &cli.subcommand {
        cli::Command::Backup(subcmd) => run::backup::run_backup(&cli, subcmd),
        cli::Command::Config(subcmd) => run::config::run_config(&cli, subcmd),
        cli::Command::Log(subcmd) => run::log::run_log(&cli, subcmd),
        cli::Command::Summary => run::summary::run_summary(&cli),
    }
}

