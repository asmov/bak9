use std::{fmt::Display, fs, path::{Path, PathBuf}, str::FromStr, sync::OnceLock};
use chrono;
use crate::{error::*, cmd::rsync, config::*, paths, schedule::*};
        
pub fn hostname() -> &'static str {
    static HOSTNAME: OnceLock<String> = OnceLock::new();
    &HOSTNAME.get_or_init(|| whoami::fallible::hostname().unwrap())
}

pub fn username() -> &'static str {
    static USERNAME: OnceLock<String> = OnceLock::new();
    &USERNAME.get_or_init(|| whoami::username())
}

pub struct BackupRunName {
    pub datetime: chrono::DateTime<chrono::Local>,
    pub hostname: String,
    pub username: String,
    pub backup_name: String,
}

impl BackupRunName {
    pub fn new(datetime: chrono::DateTime<chrono::Local>, hostname: &str, username: &str, backup_name: &str) -> Self {
        Self {
            datetime,
            hostname: hostname.to_string(),
            username: username.to_string(),
            backup_name: backup_name.to_string(),
        }
    }
}

impl Display for BackupRunName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{datetimestamp}__{hostname}__{username}__{backup_name}",
            datetimestamp = datetimestamp(self.datetime),
            hostname = self.hostname,
            username = self.username,
            backup_name = self.backup_name)
    }
}

impl From<BackupRunName> for String {
    fn from(run_name: BackupRunName) -> String {
        run_name.to_string()
    }
}

impl FromStr for BackupRunName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.split("__").collect::<Vec<&str>>();
        let datetime = from_datetimestamp(parts[0]);
        let hostname = parts[1];
        let username = parts[2];
        let backup_name = parts[3];
        Ok(Self::new(datetime, hostname, username, backup_name))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BackupType {
    Full,
    Incremental,
}

impl BackupType {
    fn subdir_name(&self) -> &'static str {
        match self {
            BackupType::Full => paths::BACKUP_FULL_DIRNAME,
            BackupType::Incremental => paths::BACKUP_INCREMENTAL_DIRNAME,
        }
    }
}

pub fn find_last_backup(backup_type: BackupType, hostname: &str, username: &str, backup_name: &str, backup_storage_dir: &Path) -> Option<PathBuf> {
    let backup_dir = backup_storage_dir.join(backup_type.subdir_name());

    let mut entries = fs::read_dir(&backup_dir).unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| {
            if !entry.metadata().is_ok_and(|metadata| metadata.is_dir()) {
                return false;
            }

            let run_name = match BackupRunName::from_str(entry.file_name().to_str().unwrap()) {
                Ok(run_name) => run_name,
                Err(_) => return false,
            };

            run_name.username == username && run_name.hostname == hostname && run_name.backup_name == backup_name
        })
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()).reverse());
    
    entries.first()
        .and_then(|entry| Some(entry.path().to_path_buf()))
}

pub type BackupJobResults = Result<Vec<BackupJobOutput>>;

pub enum BackupJobOutput {
    Full (BackupJobOutputFull),
    Incremental (BackupJobOutputIncremental)
}

pub trait BackupJobOutputImpl {
    fn name(&self) -> &str;
    fn source_dir(&self) -> &Path;
    fn dest_dir(&self) -> &Path;
}

impl BackupJobOutputImpl for BackupJobOutput {
    fn name(&self) -> &str {
        match self {
            BackupJobOutput::Full(output) => output.name(),
            BackupJobOutput::Incremental(output) => output.name(),
        }
    }

    fn source_dir(&self) -> &Path {
        match self {
            BackupJobOutput::Full(output) => output.source_dir(),
            BackupJobOutput::Incremental(output) => output.source_dir(),
        }
    }

    fn dest_dir(&self) -> &Path {
        match self {
            BackupJobOutput::Full(output) => output.dest_dir(),
            BackupJobOutput::Incremental(output) => output.dest_dir(),
        }
    }
}

pub struct BackupJobOutputFull {
    pub name: String,
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf
}

impl BackupJobOutputImpl for BackupJobOutputFull {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_dir(&self) -> &Path {
        &self.source_dir
    }

    fn dest_dir(&self) -> &Path {
        &self.dest_dir
    }
}

pub struct BackupJobOutputIncremental {
    pub name: String,
    pub source_dir: PathBuf,
    pub full_dir: PathBuf,
    pub dest_dir: PathBuf
}

impl BackupJobOutputImpl for BackupJobOutputIncremental {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_dir(&self) -> &Path {
        &self.source_dir
    }

    fn dest_dir(&self) -> &Path {
        &self.dest_dir
    }
}

/// Performs a full backup. Returns the path to the backup directory created.
pub(crate) fn backup_full(cfg_backup: &BackupConfigBackup, config: &BackupConfig) -> Result<BackupJobOutput> {
    let run_name = BackupRunName::new(datetime_now(), hostname(), username(), &cfg_backup.name);
    let source_dir = cfg_backup.source_dir_path();
    let dest_dir = config.backup_storage_dir_path()
        .join(paths::BACKUP_FULL_DIRNAME)
        .join(run_name.to_string());

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

/// Performs an incremental backup. Returns the path to the backup directory created.
pub(crate) fn backup_incremental(cfg_backup: &BackupConfigBackup, config: &BackupConfig) -> Result<BackupJobOutput> {
    let source_dir = cfg_backup.source_dir_path();
    let run_name = BackupRunName::new(datetime_now(), hostname(), username(), &cfg_backup.name);
    let dest_dir = config.backup_storage_dir_path()
        .join(paths::BACKUP_INCREMENTAL_DIRNAME)
        .join(run_name.to_string());

    let last_full_dir = find_last_backup(
            BackupType::Full,
            hostname(),
            username(),
            &cfg_backup.name,
            &config.backup_storage_dir_path()
        ).ok_or_else(|| Error::Generic("No full backup found".to_string()))?;


    let mut rsync_cmd = rsync::cmd_rsync_incremental(&last_full_dir, &source_dir, &dest_dir);
    let output = rsync_cmd.output().unwrap();

    if !output.status.success() {
        return Err(Error::Generic("TODO: RSYNC FAILED".to_string()));
    }

    Ok(BackupJobOutput::Incremental(BackupJobOutputIncremental {
        name: cfg_backup.name.clone(),
        source_dir,
        full_dir: last_full_dir,
        dest_dir,
    }))
}
 
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum BackupJob {
    Full,
    Incremental,
    //Archive,
    //SyncFull,
    //SyncIncremental,
    //SyncArchive,
}

/// Check to see if it's time to run a backup.
pub(crate) fn backup_job_due(
    cfg_backup: &BackupConfigBackup,
    config: &BackupConfig,
) -> Result<Option<BackupJob>> {
    let last_full_backup = find_last_backup(
        BackupType::Full,
        hostname(),
        username(),
        &cfg_backup.name,
        &config.backup_storage_dir_path());

    let last_full_backup = match last_full_backup {
        Some(path) => path,
        None => return Ok(Some(BackupJob::Full)),
    };

    let last_full_dirname = last_full_backup.file_name().unwrap().to_str().unwrap();
    let last_full_datetime = BackupRunName::from_str(&last_full_dirname).unwrap().datetime;
    let next_full_datetime = cron::Schedule::from(config.schedule(&cfg_backup.full_schedule).unwrap())
        .after(&last_full_datetime)
        .next()
        .unwrap();

    if next_full_datetime <= chrono::Local::now() {
        return Ok(Some(BackupJob::Full))
    }

    let last_incremental = find_last_backup(
        BackupType::Incremental,
        hostname(),
        username(),
        &cfg_backup.name,
        &config.backup_storage_dir_path());
    
    let after_datetime = if let Some(last_incremental_dir) = last_incremental {
        let last_incremental_dirname = last_incremental_dir.file_name().unwrap().to_str().unwrap();
        BackupRunName::from_str(&last_incremental_dirname).unwrap().datetime
    } else {
        last_full_datetime
    };

    let next_incremental_datetime = cron::Schedule::from(config.schedule(&cfg_backup.incremental_schedule).unwrap())
        .after(&after_datetime)
        .next()
        .unwrap();

    if next_incremental_datetime <= chrono::Local::now() {
        Ok(Some(BackupJob::Incremental))
    } else {
        Ok(None)
    }
}






