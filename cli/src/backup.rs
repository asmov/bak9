mod plan;

use std::{fmt::Display, fs, path::{Path, PathBuf}, str::FromStr, sync::OnceLock};
use chrono;
use strum;
use crate::{archive::*, cmd::rsync, config::*, error::*, job::*, log::*, paths::*, schedule::*, sync::*};
        
pub fn hostname() -> &'static str {
    static HOSTNAME: OnceLock<String> = OnceLock::new();
    &HOSTNAME.get_or_init(|| whoami::fallible::hostname().unwrap())
}

pub fn username() -> &'static str {
    static USERNAME: OnceLock<String> = OnceLock::new();
    &USERNAME.get_or_init(|| whoami::username())
}

#[derive(Debug, Clone)]
pub struct BackupRunName {
    pub datetime: chrono::DateTime<chrono::Local>,
    pub hostname: String,
    pub username: String,
    pub backup_name: String,
    str: String,
}

impl BackupRunName {
    pub fn new(datetime: chrono::DateTime<chrono::Local>, hostname: &str, username: &str, backup_name: &str) -> Self {
        let str = format!("{datetimestamp}__{hostname}__{username}__{backup_name}",
            datetimestamp = datetimestamp(datetime),
            hostname = hostname,
            username = username,
            backup_name = backup_name);

        Self {
            datetime,
            hostname: hostname.to_string(),
            username: username.to_string(),
            backup_name: backup_name.to_string(),
            str,
        }
    }

    fn as_str(&self) -> &str {
        self.str.as_str()
    }
}

impl Display for BackupRunName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str.as_str())
    }
}

impl AsRef<Path> for BackupRunName {
    fn as_ref(&self) -> &Path {
        Path::new(&self.str)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Debug)]
pub struct BackupJob {
    pub(crate) backup_type: BackupType,
    pub(crate) run_name: BackupRunName,
    pub(crate) source_dir: PathBuf,
    pub(crate) incremental_source_dir: Option<PathBuf>,
    pub(crate) dest_dir: PathBuf,
}

impl JobTrait for BackupJob {
    type Output = BackupJobOutput;

    fn run(&self) -> Result<JobOutput> {
        log_info!("Began {} backup of {}", self.backup_type, self.run_name.backup_name.tik_name());

        let mut rsync_cmd = match self.backup_type {
            BackupType::Full => rsync::cmd_rsync_full(&self.source_dir, &self.dest_dir),
            BackupType::Incremental => rsync::cmd_rsync_incremental(
                &self.source_dir,
                &self.incremental_source_dir.as_ref().unwrap(),
                &self.dest_dir),
        };

        let output = rsync_cmd.output().unwrap();

        if !output.status.success() {
            return Err(Error::rsync(output));
        }
    
        log_info!("Completed {} backup of {} to {}",
            self.backup_type, self.run_name.backup_name.tik_name(), self.dest_dir.to_str().unwrap().tik_path());

        Ok(JobOutput::Backup(BackupJobOutput {
            backup_type: self.backup_type,
            run_name: self.run_name.clone(),
            source_dir: self.source_dir.clone(),
            incremental_source_dir: self.incremental_source_dir.clone(),
            dest_dir: self.dest_dir.clone(),
        }))
    }
}

#[derive(Debug)]
pub struct BackupJobOutput {
    pub backup_type: BackupType,
    pub run_name: BackupRunName,
    pub source_dir: PathBuf,
    pub incremental_source_dir: Option<PathBuf>,
    pub dest_dir: PathBuf,
}

impl JobOutputTrait for BackupJobOutput {}


fn find_last_backup<P: AsRef<Path>>(
    backup_type: BackupType,
    hostname: &str,
    username: &str,
    backup_name: &str,
    backup_storage_dir: P
) -> Option<Bak9Path> {
    let backup_dir = Bak9Path::backup_dir(&backup_storage_dir,
        BackupPathParts::new(backup_type, hostname, username, backup_name));

    let mut backup_runs = fs::read_dir(&backup_dir).unwrap()
        .map(|entry| entry.unwrap())
        .filter_map(|entry| {
            if !entry.metadata().is_ok_and(|metadata| metadata.is_dir()) {
                return None;
            }

            match BackupRunName::from_str(entry.file_name().to_str().unwrap()) {
                Ok(run_name) if run_name.username == username
                        && run_name.hostname == hostname
                        && run_name.backup_name == backup_name => Some(run_name),
                _ => None
            }
        })
        .collect::<Vec<_>>();

    backup_runs.sort_by(|a, b| a.as_str().cmp(b.as_str()).reverse());
    backup_runs.first()
        .and_then(|backup_run_name| Some(Bak9Path::backup(&backup_storage_dir, backup_type, backup_run_name)))
}


/// Check to see if it's time to run a backup.
pub(crate) fn backup_job_due(
    cfg_backup: &BackupConfigBackup,
    config: &BackupConfig,
) -> Result<Option<JobQueueEntry>> {
    let last_full_backup = find_last_backup(
        BackupType::Full,
        hostname(),
        username(),
        &cfg_backup.name,
        &config.backup_storage_dir_path());

    let last_full_backup = match last_full_backup {
        Some(bak9path) => bak9path,
        None => return Ok(Some(BackupJob::plan(BackupType::Full, &cfg_backup, config)?))
    };

    let last_full_run_name = last_full_backup.backup_run_name().unwrap();
    let last_full_datetime = last_full_run_name.datetime;
    let next_full_datetime = cron::Schedule::from(config.schedule(&cfg_backup.full_schedule).unwrap())
        .after(&last_full_datetime)
        .next()
        .unwrap();

    if next_full_datetime <= chrono::Local::now() {
        return Ok(Some(BackupJob::plan(BackupType::Full, &cfg_backup, config)?));
    }

    let last_incremental = find_last_backup(
        BackupType::Incremental,
        hostname(),
        username(),
        &cfg_backup.name,
        &config.backup_storage_dir_path());
    
    let after_datetime = if let Some(last_incremental) = last_incremental {
        let last_incremental_dirname = last_incremental.backup_run_name().unwrap().as_str();
        BackupRunName::from_str(&last_incremental_dirname).unwrap().datetime
    } else {
        last_full_datetime
    };

    let next_incremental_datetime = cron::Schedule::from(config.schedule(&cfg_backup.incremental_schedule).unwrap())
        .after(&after_datetime)
        .next()
        .unwrap();

    if next_incremental_datetime <= chrono::Local::now() {
        Ok(Some(BackupJob::plan(BackupType::Incremental, &cfg_backup, config)?))
    } else {
        Ok(None)
    }
}
