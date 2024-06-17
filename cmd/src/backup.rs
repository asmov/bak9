use std::{fs, path::{PathBuf, Path}};
use chrono::{self, TimeZone};
use lazy_static::lazy_static;
use crate::{Error, Result, cmd::rsync, cli::Cli, config::{self, BackupConfig, BackupConfigBackup}, paths, schedule};
        
lazy_static! {
    static ref HOSTNAME: String = hostname::get().unwrap().into_string().unwrap();
}

pub(crate) fn hostname() -> &'static str {
    &HOSTNAME
}

pub fn backup_run_name(timestamp: &str, backup_name: &str, host: &str) -> String {
    format!("{timestamp}_{backup_name}_{host}")
}

pub fn parse_backup_name<'filename>(
    filename: &'filename str,
    _config: &config::BackupConfig
) -> (chrono::DateTime<chrono::Local>, &'filename str, &'filename str) {
    let parts = filename.split('_').collect::<Vec<&str>>();
    //todo: determine timestamp format
    let timestamp = chrono::NaiveDateTime::parse_from_str(parts[0], schedule::DATETIMESTAMP_FORMAT).unwrap();
    let timestamp = chrono::Local.from_local_datetime(&timestamp).unwrap();
    let backup_name = parts[1];
    let host = parts[2];
    (timestamp, backup_name, host)
}

pub fn find_last_full_backup(backup_name: &str, host: &str, backup_storage_dir: &Path) -> Option<PathBuf> {
    let backup_full_dir = backup_storage_dir.join(paths::BACKUP_FULL_DIRNAME);
    let filename_ending = format!("_{backup_name}_{host}");
    let mut entries = fs::read_dir(&backup_full_dir).unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| {
            entry.metadata().unwrap().is_dir()
            && entry.file_name().to_str().unwrap().ends_with(&filename_ending)
        })
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()).reverse());
    
    entries.first()
        .and_then(|entry| Some(entry.path().to_path_buf()))
}

pub type BackupJobResults = Result<Vec<BackupJobOutput>>;

pub enum BackupJobOutput {
    Full (BackupJobOutputFull),
}

pub struct BackupJobOutputFull {
    pub name: String,
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf,
}


/// Run a backup. Returns the path to
pub(crate) fn backup_full(cfg_backup: &BackupConfigBackup, config: &BackupConfig) -> Result<BackupJobOutput> {
        let run_name = backup_run_name(&schedule::datetimestamp_today(), &cfg_backup.name, hostname());
        let source_dir = cfg_backup.source_dir_path();
        let dest_dir = config.backup_storage_dir_path()
            .join(paths::BACKUP_FULL_DIRNAME)
            .join(&run_name);

        let mut rsync_cmd = rsync::cmd_rsync_full(&source_dir, &dest_dir);
        let output = rsync_cmd.output().unwrap();

        if !output.status.success() {
            return Err(Error::Generic("TODO: RSYNC FAILED".to_string()));
        }

        Ok(BackupJobOutput::Full(BackupJobOutputFull {
            name: cfg_backup.name.clone(),
            source_dir,
            dest_dir,
        }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackupJob {
    Full,
    Incremental,
    Archive,
    SyncFull,
    SyncIncremental,
    SyncArchive,
}


/// Check to see if it's time to run a backup.
pub(crate) fn backup_job_due(
    cfg_backup: &BackupConfigBackup,
    config: &BackupConfig,
) -> Result<Option<BackupJob>> {
    let last_full_backup = find_last_full_backup(
        &cfg_backup.name,
        hostname(),
        &config.backup_storage_dir_path());
    let last_full_backup = match last_full_backup {
        Some(path) => path,
        None => return Ok(Some(BackupJob::Full)),
    };

    let last_filename = last_full_backup.file_name().unwrap().to_str().unwrap();
    let (last_backup_time, _, _) = parse_backup_name(&last_filename, &config);
    let schedule_cfg = config.schedule(&cfg_backup.incremental_schedule).unwrap();
    let schedule = cron::Schedule::from(schedule_cfg);
    let next = schedule.after(&last_backup_time).next().unwrap();

    if next <= chrono::Local::now() {
        Ok(Some(BackupJob::Full))
    } else {
        Ok(None)
    }
}





