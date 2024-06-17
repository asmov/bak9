use std::{path::PathBuf, process};
use crate::{backup::*, Error, schedule, cmd::rsync, cli::*, config::*, paths, Result};

pub fn run_backup(cli: &Cli, subcmd: &BackupCommand, config: Option<&BackupConfig>) -> BackupJobResults {
    let config = select_config!(cli, config);
    verify_environment(&config)?;

    match subcmd {
        BackupCommand::Scheduled => run_backup_scheduled(cli, &config),
    }
}

fn run_backup_scheduled(cli: &Cli, config: &BackupConfig) -> BackupJobResults {
    let mut jobs = Vec::new();
    for cfg_backup in &config.backups {
        if let Some(job) = backup_job_due(&cfg_backup, &config)? {
            jobs.push((cfg_backup, job));
        }
    }

    let mut results = Vec::new();
    for (cfg_backup, job) in jobs {
        match job {
            BackupJob::Full => {
                let output = backup_full(&cfg_backup, &config)?;
                results.push(output);
            },
            _ => todo!(),
        }
    }

    Ok(results)
}

/// Verify that the runtime environment that has been configured is valid.  
/// Verification:
/// - Directories need to exist
fn verify_environment(config: &BackupConfig) -> Result<()> {
    paths::verify_backup_dirs(config)?;
    Ok(())
}

