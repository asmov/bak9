use std::{path::PathBuf, process};
use crate::{backup::backup_full, Error, schedule, cmd::rsync,
    cli::{Cli, BackupCommand}, config::{read_cli_config, BackupConfig, BackupConfigBackup}, paths, Result};

pub(crate) fn run_backup(cli: &Cli, subcmd: &BackupCommand) -> Result<process::ExitCode> {
    let config = read_cli_config(cli)?;
    verify_environment(&config)?;

    match subcmd {
        BackupCommand::Scheduled => run_backup_scheduled(cli, &config),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackupJob {
    Full,
    Incremental,
    Archive,
    SyncFull,
    SyncIncremental,
    SyncArchive,
}

fn run_backup_scheduled(cli: &Cli, config: &BackupConfig) -> Result<process::ExitCode> {
    let mut jobs = Vec::new();
    for cfg_backup in &config.backups {
        if let Some(job) = backup_job_due(&cfg_backup, &config, &cli)? {
            jobs.push((cfg_backup, job));
        }
    }

    for (cfg_backup, job) in jobs {
        match job {
            BackupJob::Full => {
                backup_full(&cfg_backup, &config, &cli)?;
            },
            _ => todo!(),
        }
    }

    Ok(process::ExitCode::SUCCESS)
}

/// Check to see if it's time to run a backup.
fn backup_job_due(
    cfg_backup: &BackupConfigBackup,
    config: &BackupConfig,
    cli: &Cli
) -> Result<Option<BackupJob>> {
    todo!()
}

/// Verify that the runtime environment that has been configured is valid.  
/// Verification:
/// - Directories need to exist
fn verify_environment(config: &BackupConfig) -> Result<()> {
    paths::verify_backup_dirs(config)?;
    Ok(())
}

