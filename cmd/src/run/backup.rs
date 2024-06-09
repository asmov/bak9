use std::process;
use crate::{paths, config, cli, Result};

pub fn run_backup(cli: &cli::Cli, subcmd: &cli::BackupCommand) -> Result<process::ExitCode> {
    let config = config::read_config(cli.config_file.as_ref())?;
    verify_environment(&config)?;

    match subcmd {
        cli::BackupCommand::Scheduled => run_backup_scheduled(cli),
    }
}

pub fn run_backup_scheduled(cli: &cli::Cli) -> Result<process::ExitCode> {
    dbg!(cli);
    todo!()
}

/// Verify that the runtime environment that has been configured is valid.  
/// Verification:
/// - Directories need to exist
pub fn verify_environment(config: &config::BackupConfig) -> Result<()> {
    paths::verify_backup_dirs(config)?;

    Ok(())
}

