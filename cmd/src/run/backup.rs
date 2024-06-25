use crate::{backup::*, cli::*, config::*, log::*, paths, error::*, job::*};

pub fn run_backup(cli: &Cli, subcmd: &BackupCommand, config: Option<&BackupConfig>) -> JobResults {
    let config = select_config!(cli, config);
    verify_environment(&config)?;

    match subcmd {
        BackupCommand::Scheduled => run_backup_scheduled(cli, &config),
        BackupCommand::Full(cmd) => run_backup_manual(cli, cmd, &config, BackupType::Full),
        BackupCommand::Incremental(cmd) => run_backup_manual(cli, cmd, &config, BackupType::Incremental),
    }
}

fn run_backup_scheduled(_cli: &Cli, config: &BackupConfig) -> JobResults {
    let mut jobs = JobQueue::new();
    for cfg_backup in &config.backups {
        if let Some(queue_entry) = backup_job_due(&cfg_backup, &config)? {
            jobs.push(queue_entry);
        }
    }

    if jobs.is_empty() {
        Log::get().info("No backups are due");
        return Ok(Vec::new());
    }

    run_jobs(jobs, &config)
}

const ALL: &str = "all";

fn run_backup_manual(
    _cli: &Cli,
    cmd: &ManualBackupCommand,
    config: &BackupConfig,
    backup_type: BackupType
) -> JobResults {
    let mut cfg_backups = Vec::new();
    if cmd.name == ALL {
        cfg_backups.extend(config.backups.iter());
    } else {
        let cfg_backup = config.backup(&cmd.name)?;
        cfg_backups.push(cfg_backup);
    };
    
    let mut results = Vec::new();
    for cfg_backup in cfg_backups {
        let jobs = vec![BackupJob::plan(backup_type, &cfg_backup, &config)];
        let job_results = run_jobs(jobs, config)?;
        results.extend(job_results);
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

