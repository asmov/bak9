use std::{fs, path::{Path, PathBuf}};
use chrono::{self, TimeZone};
use serde;
use thiserror;
use toml;
use validator::{Validate, ValidationError};
use hostname;
use cron;

const HOME_CONFIG_DIR: &'static str = ".config/asmov/dewdrop/backup";
const CONFIG_FILENAME: &'static str = "config.toml";

const BACKUP_ARCHIVE_DIRNAME: &'static str = "archive";
const BACKUP_FULL_DIRNAME: &'static str = "full";
const BACKUP_INCREMENTAL_DIRNAME: &'static str = "incremental";
const BACKUP_LOGS_DIRNAME: &'static str = "logs";

const DATESTAMP_FORMAT: &'static str = "%Y-%m-%d";
const DATETIMESTAMP_FORMAT: &'static str = "%Y-%m-%d-%H-%M-%S-%3f";


const CONFIG_DEFAULTS: &'static str = r#"
backup_storage_dir = "/storage/backup"

[[schedule]]
name = "daily"
minute = 30
hour = 2

[[schedule]]
name = "weekly"
minute = 30
hour = 2
day_of_week = "sun"

[[schedule]]
name = "monthly"
minute = 30
hour = 2
day_of_month = 1

[[schedule]]
name = "quarterly"
minute = 30
hour = 2
day_of_month = 1
months = [1, 4, 7, 10]

[[schedule]]
name = "annual"
minute = 30
hour = 2
day_of_month = 1
month = 1

[[backup]]
name = "home-$USER"
source_dir = "$HOME"
full_schedule = "monthly"
incremental_schedule = "daily"
max_full = 3

[[backup.archive]]
schedule = "quarterly"
max_archives = "4"

[[backup.archive]]
schedule = "annual"
max_archives = "3"
"#;

const RSYNC_CMD: &'static str = "rsync";
const RSYNC_FLAG_ARCHIVE: &'static str = "--archive";
/// Hard links files from LINKSRC to DEST when they are identical to their counterpart in SOURCE, rather than copying
const RSYNC_FLAG_LINK_DESTINATION: &'static str = "--link-dest";
/// Does not place a file in DEST that no longer exists in SOURCE, regardless of whether it exists in LINKSRC
const RSYNC_FLAG_DELETE: &'static str = "--delete";

const TEST_HOME_DIR: &'static str = "tests/fixtures/fs/home/testusr";
const TEST_VAR_CARGO_TARGET_TMPDIR: &'static str = "$CARGO_TARGET_TMPDIR";
const TEST_VAR_CARGO_MANIFEST_DIR: &'static str = "$CARGO_MANIFEST_DIR";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config file error: {path} :> {cause}")]
    ConfigFile { path: String, cause: String },

    #[error("File IO error: {path} :> {cause}")]
    FileIO{ path: String, cause: String },
    
    #[error("Config {schema} not found: {identifier}")]
    ConfigNotFound { schema: &'static str, identifier: String },
}

impl Error {
    pub fn new_config_file(path: &Path, err: impl std::error::Error) -> Self {
        Error::ConfigFile {
            path: path.to_str().unwrap().to_string(),
            cause: err.to_string()
        }
    }

    pub fn new_config(path: &Path, config: impl std::fmt::Display, msg: &str) -> Self {
        Error::ConfigFile {
            path: path.to_str().unwrap().to_string(),
            cause: format!("{msg} :: {config}")
        }
    }

    pub fn new_file_io(path: &Path, err: impl std::error::Error) -> Self {
        Error::FileIO {
            path: path.to_str().unwrap().to_string(),
            cause: err.to_string()
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
#[validate]
pub struct BackupConfig {
    pub backup_storage_dir: String,
    #[serde(alias = "schedule")]
    #[validate(nested)]
    pub schedules: Vec<BackupConfigSchedule>,
    #[serde(alias = "backup")]
    #[validate(nested)]
    pub backups: Vec<BackupConfigBackup>,
}

impl BackupConfig {
    pub fn read(filepath: &Path) -> Result<Self> {
        let content = fs::read_to_string(filepath)
            .map_err(|e| Error::new_config_file(filepath, e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| Error::new_config_file(filepath, e))?;

        config.validate()
            .map_err(|e| Error::new_config_file(filepath, e))?;

        Ok(config)
    }

    pub fn read_home() -> Result<Self> {
        let config_filepath = home_dir()?
            .join(HOME_CONFIG_DIR)
            .join(CONFIG_FILENAME);
        Self::read(&config_filepath)
    }

    pub fn backup_storage_dir_path(&self) -> PathBuf {
        config_path(&self.backup_storage_dir)
    }

    pub fn schedule<'cfg>(&'cfg self, schedule_name: &str) -> Result<&'cfg BackupConfigSchedule> {
        self.schedules.iter()
            .find(|s| s.name == schedule_name)
            .ok_or_else(|| Error::ConfigNotFound { schema: "Schedule", identifier: schedule_name.to_string() })
    }

    pub fn backup<'cfg>(&'cfg self, backup_name: &str) -> Result<&'cfg BackupConfigBackup> {
        self.backups.iter()
            .find(|b| b.name == backup_name)
            .ok_or_else(|| Error::ConfigNotFound { schema: "Backup", identifier: backup_name.to_string() })
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum DayOfWeek {
    #[serde(alias="sun")]
    Sunday = 0,
    #[serde(alias="mon")]
    Monday = 1,
    #[serde(alias="tue")]
    Tuesday = 2,
    #[serde(alias="wed")]
    Wednesday = 3,
    #[serde(alias="thu")]
    Thursday = 4,
    #[serde(alias="fri")]
    Friday = 5,
    #[serde(alias="sat")]
    Saturday = 6
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
#[validate(schema(function = "BackupConfigSchedule::validate_schema"))]
pub struct BackupConfigSchedule {
    pub name: String,
    pub minute: Option<u8>,
    pub minutes: Option<Vec<u8>>,
    pub hour: Option<u8>,
    pub hours: Option<Vec<u8>>,
    pub day_of_week: Option<DayOfWeek>,
    pub days_of_week: Option<Vec<DayOfWeek>>,
    pub day_of_month: Option<u8>,
    pub days_of_month: Option<Vec<u8>>,
    pub month: Option<u8>,
    pub months: Option<Vec<u8>>
}

impl BackupConfigSchedule {
    pub fn validate_schema(&self) -> std::result::Result<(), validator::ValidationError> {
        if self.month.is_some() && self.months.is_some() {
            Err(ValidationError::new("'month' and months are mutually exclusive"))
        } else if self.day_of_month.is_some() && self.days_of_month.is_some() {
            Err(ValidationError::new("'day_of_month' and 'days_of_month' are mutually exclusive"))
        } else if self.day_of_week.is_some() && self.days_of_week.is_some() {
            Err(ValidationError::new("'day_of_week' and 'days_of_week' are mutually exclusive"))
        } else if self.hour.is_some() && self.hours.is_some() {
            Err(ValidationError::new("'hour' and 'hours' are mutually exclusive"))
        } else if self.minute.is_some() && self.minutes.is_some() {
            Err(ValidationError::new("'minute' and 'minutes' are mutually exclusive"))
        } else {
            Ok(())
        }
    }

    pub fn to_cron_expression(&self) -> String {
        let mut cron = String::new();

        // seconds
        cron.push_str("0 ");

        if let Some(minutes) = &self.minutes {
            let minutes = minutes.iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(",");
            cron.push_str(&minutes);
        } else if let Some(minute) = self.minute {
            cron.push_str(&minute.to_string());
        } else {
            cron.push('*');
        }

        cron.push(' ');

        if let Some(hours) = &self.hours {
            let hours = hours.iter()
                .map(|h| h.to_string())
                .collect::<Vec<String>>()
                .join(",");
            cron.push_str(&hours);
        } else if let Some(hour) = self.hour {
            cron.push_str(&hour.to_string());
        } else {
            cron.push('*');
        }

        cron.push(' ');

        if let Some(days_of_month) = &self.days_of_month {
            let days_of_month = days_of_month.iter()
                .map(|d| d.to_string())
                .collect::<Vec<String>>()
                .join(",");
            cron.push_str(&days_of_month);
        } else if let Some(day_of_month) = self.day_of_month {
            cron.push_str(&day_of_month.to_string());
        } else {
            cron.push('*');
        }

        cron.push(' ');

        if let Some(months) = &self.months {
            let months = months.iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(",");
            cron.push_str(&months);
        } else if let Some(month) = self.month {
            cron.push_str(&month.to_string());
        } else {
            cron.push('*');
        }

        cron.push(' ');

        if let Some(days_of_week) = &self.days_of_week {
            let days_of_week = days_of_week.iter()
                .map(|d| (*d as u8).to_string())
                .collect::<Vec<String>>()
                .join(",");
            cron.push_str(&days_of_week);
        } else if let Some(day_of_week) = self.day_of_week {
            cron.push_str(&(day_of_week as u8).to_string());
        } else {
            cron.push('*');
        }

        // years
        cron.push_str(" *");

        cron
    }
}

// inefficient, but the only thing available from the cron crate
impl From<&BackupConfigSchedule> for cron::Schedule {
    fn from(cfg: &BackupConfigSchedule) -> Self {
        <Self as std::str::FromStr>::from_str(&cfg.to_cron_expression()).unwrap()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct BackupConfigBackup {
    #[validate(does_not_contain(pattern = "_"))]
    pub name: String,
    pub source_dir: String,
    pub full_schedule: String,
    pub incremental_schedule: String,
    pub max_full: u32,

    #[serde(alias = "archive")]
    #[validate(nested)]
    pub archives: Vec<BackupConfigArchive>

}

impl BackupConfigBackup {
    pub fn source_dir_path(&self) -> PathBuf {
        config_path(&self.source_dir)
    }

    pub fn archive<'cfg>(&'cfg self, schedule_name: &str) -> Result<&'cfg BackupConfigArchive> {
        self.archives.iter()
            .find(|b| b.schedule == schedule_name)
            .ok_or_else(|| Error::ConfigNotFound {
                schema: "Archive",
                identifier: format!("{}:{}", self.name.to_string(), schedule_name.to_string())
            })
    }
}
 
#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct BackupConfigArchive {
    pub schedule: String,
    pub max_archives: u32,
}

fn config_path(path_str: &str) -> PathBuf {
    #[cfg(not(test))]
    return PathBuf::from(&path_str);

    #[cfg(test)] {
        let path_str = path_str
            .replace(TEST_VAR_CARGO_MANIFEST_DIR, env!("CARGO_MANIFEST_DIR"))
            .replace(TEST_VAR_CARGO_TARGET_TMPDIR, env!("CARGO_TARGET_TMPDIR"));
        PathBuf::from(&path_str)
    }
}
 
pub fn home_dir() -> Result<PathBuf> {
    #[cfg(not(test))]
    let home = option_env!("HOME")
        .ok_or_else(|| Error::FileIO { path: "$HOME".to_string(), cause: "$HOME is not set".to_string() })?;

    #[cfg(test)]
    let home = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(TEST_HOME_DIR)
        .to_str().unwrap()
        .to_string();

    Ok(PathBuf::from(home))
}

pub fn setup_home_config(force: bool) -> Result<()> {
    let home_config_dir = home_dir()?.join(HOME_CONFIG_DIR);
    let home_config_file = home_config_dir.join(CONFIG_FILENAME);

    if !home_config_dir.exists() {
        fs::create_dir_all(&home_config_dir)
            .map_err(|e| Error::new_file_io(&home_config_dir, e))?;
    }

    if !home_config_file.exists() || force {
        fs::write(&home_config_file, CONFIG_DEFAULTS)
            .map_err(|e| Error::new_file_io(&home_config_file, e))?;
    }

    Ok(())
}

pub fn setup_backup_dirs(config: &BackupConfig, force: bool) -> Result<()> {
    let backup_dir = config.backup_storage_dir_path();

    if backup_dir.exists() && !force {
        return Ok(());
    }

    let dirs: [&Path; 5] = [
        &backup_dir,
        &backup_dir.join(BACKUP_ARCHIVE_DIRNAME),
        &backup_dir.join(BACKUP_FULL_DIRNAME),
        &backup_dir.join(BACKUP_INCREMENTAL_DIRNAME),
        &backup_dir.join(BACKUP_LOGS_DIRNAME)
    ];

    for dir in dirs {
        fs::create_dir_all(dir)
            .map_err(|e| Error::new_file_io(&backup_dir, e))?;
    }

    Ok(())
}

pub fn backup_run_name(timestamp: &str, backup_name: &str, host: &str) -> String {
    format!("{timestamp}_{backup_name}_{host}")
}

pub fn cmd_rsync_full(source_dir: &Path, dest_dir: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(RSYNC_CMD);
    cmd.args(&[
        RSYNC_FLAG_ARCHIVE,
        source_dir.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
    ]);

    cmd
}

pub fn cmd_rsync_incremental(previous_backup_dir: &Path, source_dir: &Path, dest_dir: &Path) -> std::process::Command {
    let hardlink_source = format!("{}/", previous_backup_dir.to_str().unwrap());

    let mut cmd = std::process::Command::new(RSYNC_CMD);
    cmd.args(&[
        RSYNC_FLAG_ARCHIVE,
        RSYNC_FLAG_DELETE,
        RSYNC_FLAG_LINK_DESTINATION,
        &hardlink_source,
        source_dir.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
    ]);

    cmd
}

// tar cf "$home_backup_zip" --use-compress-program='xz -T0' "$home_backup_name"
pub fn cmd_tar_xz(source_dir: &Path, zip_file: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("tar");
    cmd
        .current_dir(source_dir.parent().unwrap())
        .args(&[
            "cf",
            zip_file.to_str().unwrap(),
            "--use-compress-program",
            "xz -T0",
            source_dir.file_name().unwrap().to_str().unwrap(),
        ]);

    cmd
}

pub fn find_last_full_backup(backup_name: &str, host: &str, config: &BackupConfig) -> Option<PathBuf> {
    let backup_full_dir = config.backup_storage_dir_path().join(BACKUP_FULL_DIRNAME);
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

pub fn parse_backup_name<'filename>(
    filename: &'filename str,
    _config: &BackupConfig
) -> (chrono::DateTime<chrono::Local>, &'filename str, &'filename str) {
    let parts = filename.split('_').collect::<Vec<&str>>();
    //todo: determine timestamp format
    let timestamp = chrono::NaiveDateTime::parse_from_str(parts[0], DATETIMESTAMP_FORMAT).unwrap();
    let timestamp = chrono::Local.from_local_datetime(&timestamp).unwrap();
    let backup_name = parts[1];
    let host = parts[2];
    (timestamp, backup_name, host)
}

pub fn datetimestamp_today() -> String {
    chrono::Local::now()
        .format(&DATETIMESTAMP_FORMAT)
        .to_string()
}

pub fn datetimestamp_yesterday() -> String {
    let yesterday = chrono::Local::now() - chrono::Duration::days(1);
    yesterday
        .format(&DATETIMESTAMP_FORMAT)
        .to_string()
}

pub fn datestamp_today() -> String {
    chrono::Local::now()
        .format(DATESTAMP_FORMAT)
        .to_string()
}

pub fn datestamp_yesterday() -> String {
    let yesterday = chrono::Local::now() - chrono::Duration::days(1);
    yesterday
        .format(DATESTAMP_FORMAT)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::MetadataExt;

    use chrono::NaiveTime;

    //use asmov_dewdrop_backup::*;
    use super::*;

    fn read_config() -> BackupConfig {
        let config_path = home_dir().unwrap()
            .join(HOME_CONFIG_DIR)
            .join(CONFIG_FILENAME);

        BackupConfig::read(&config_path).unwrap()
    }

    const TEST_CONFIG_BACKUP_DIR: &'static str = "$CARGO_TARGET_TMPDIR/strg/backup";

    fn cmd_diff(source_dir: &Path, dest_dir: &Path) -> std::process::Command {
        let mut cmd = std::process::Command::new("diff");
        let dest_dir = dest_dir.join(source_dir.file_name().unwrap());
        cmd.args(&[
            "-q",
            source_dir.to_str().unwrap(),
            dest_dir.to_str().unwrap(),
        ]);
        cmd
    }

    // verify that all files were hardlinked from the previous backup
    fn verify_hardlinked(dir: &Path) {
        for entry in fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_file() {
                assert!(metadata.nlink() > 1);
            } else {
                verify_hardlinked(&entry.path());
            }
        }
    }

    fn _setup_tmp_dirs(config: &BackupConfig) {
        setup_backup_dirs(config, true).unwrap();
    }

    fn wipe_tmp_dirs() {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("strg");
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
    }

    #[test]
    fn test_read_config() {
        let config = read_config();
        dbg!(&config);
        assert_eq!(config.backup_storage_dir, TEST_CONFIG_BACKUP_DIR);
        assert_eq!(config.backup("home-testusr").unwrap().archives.len(), 2);
        assert_eq!(config.backup("home-testusr").unwrap().archive("quarterly").unwrap().max_archives, 4);
    }

    #[test]
    fn test_setup_backup_dirs() {
        let config = read_config();
        setup_backup_dirs(&config, true).unwrap();
        assert!(config.backup_storage_dir_path().exists());
        assert!(config.backup_storage_dir_path().join(BACKUP_ARCHIVE_DIRNAME).exists());
        assert!(config.backup_storage_dir_path().join(BACKUP_FULL_DIRNAME).exists());
        assert!(config.backup_storage_dir_path().join(BACKUP_INCREMENTAL_DIRNAME).exists());
        assert!(config.backup_storage_dir_path().join(BACKUP_LOGS_DIRNAME).exists());
    }

    fn do_backup_full(name: &str, config: &BackupConfig, host: &str) -> PathBuf {
        let backup_cfg = config.backup(name).unwrap();
        let run_name = backup_run_name(&datetimestamp_today(), &backup_cfg.name, &host);
        let source_dir = backup_cfg.source_dir_path();
        let dest_dir = config.backup_storage_dir_path()
            .join(BACKUP_FULL_DIRNAME)
            .join(&run_name);

        // perform a full backup
        let mut rsync_cmd = cmd_rsync_full(&source_dir, &dest_dir);
        let started = std::time::Instant::now();
        let output = rsync_cmd.output().unwrap();
        dbg!(&output);
        assert!(output.status.success());

        // verify that the full backup is correct
        do_diff(&source_dir, &dest_dir);

        if started.elapsed().as_millis() < 1 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        dest_dir
    }

    // perform an incremental backup off of the previous one
    fn do_backup_incremental(name: &str, config: &BackupConfig, host: &str, prev_full_dir: &Path) -> PathBuf {
        let backup_cfg = config.backup(name).unwrap();
        let source_dir = backup_cfg.source_dir_path();
        let run_name = backup_run_name(&datetimestamp_today(), &backup_cfg.name, &host);
        let dest_dir = config.backup_storage_dir_path()
            .join(BACKUP_INCREMENTAL_DIRNAME)
            .join(&run_name);
        let mut rsync_cmd = cmd_rsync_incremental(prev_full_dir, &source_dir, &dest_dir);
        let started = std::time::Instant::now();
        let output = rsync_cmd.output().unwrap();
        dbg!(&output);
        assert!(output.status.success());

        // verify that the incremental backup is correct
        do_diff(&source_dir, &dest_dir);

        // make sure that all files were hardlinked on the last incremental run
        verify_hardlinked(&dest_dir);
        
        if started.elapsed().as_millis() < 1 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        dest_dir
    }

    fn do_zip(backup_name: &str, config: &BackupConfig, host: &str, full_dir: &Path) -> PathBuf {
        let backup_cfg = config.backup(backup_name).unwrap();
        let _source_dir = backup_cfg.source_dir_path();
        let run_name = backup_run_name(&datetimestamp_today(), &backup_cfg.name, &host);
         let zip_file = config.backup_storage_dir_path()
            .join(BACKUP_ARCHIVE_DIRNAME)
            .join(&run_name)
            .with_extension("tar.xz");
        let mut tar_xz_cmd = cmd_tar_xz(&full_dir, &zip_file);
        let output = tar_xz_cmd.output().unwrap();
        dbg!(&output);
        assert!(output.status.success());

        //todo: unzip and diff the archive against the full backup

        zip_file
    }

    fn do_diff(source_dir: &Path, dest_dir: &Path) {
        let mut diff_cmd = cmd_diff(&source_dir, &dest_dir);
        let output = diff_cmd.output().unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_backup_and_archive_mechanics() {
        let config = read_config();
        wipe_tmp_dirs();
        setup_backup_dirs(&config, true).unwrap();
        let host = hostname::get().unwrap().into_string().unwrap();

        for backup_cfg in &config.backups {
            let backup_name = &backup_cfg.name;
            let full_dir = do_backup_full(&backup_name, &config, &host);

            // zip the full backup
            do_zip(&backup_name, &config, &host, &full_dir);

            // ensure that the timestamp is always different

            // perform an incremental backup off of the previous one
            do_backup_incremental(&backup_name, &config, &host, &full_dir);
        }
    }

    #[test]
    fn test_scheduling_logic() {
        let config = read_config();
        let host = hostname::get().unwrap().into_string().unwrap();
        wipe_tmp_dirs();
        setup_backup_dirs(&config, true).unwrap();
        let tomorrow = chrono::Local::now() + chrono::Duration::days(1);
        let expected_next = tomorrow
            .with_time(NaiveTime::from_hms_opt(2, 30, 0).unwrap()).unwrap();

        for backup_cfg in &config.backups {
            do_backup_full(&backup_cfg.name, &config, &host);

            let last_full_backup = find_last_full_backup(&backup_cfg.name, &host, &config).unwrap();
            let last_filename = last_full_backup.file_name().unwrap().to_str().unwrap();
            let (last_backup_time, _, _) = parse_backup_name(&last_filename, &config);

            let schedule_cfg = config.schedule(&backup_cfg.incremental_schedule).unwrap();
            let schedule = cron::Schedule::from(schedule_cfg);
            let next = schedule.after(&last_backup_time).next().unwrap();
            assert_eq!(next, expected_next);
        }
    }
}