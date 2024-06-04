use crate::{config, schedule, paths};
use std::{fs, path::PathBuf};
use chrono::{self, TimeZone};

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

pub fn find_last_full_backup(backup_name: &str, host: &str, config: &config::BackupConfig) -> Option<PathBuf> {
    let backup_full_dir = config.backup_storage_dir_path().join(paths::BACKUP_FULL_DIRNAME);
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




