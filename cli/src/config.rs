use std::{fs, path::{Path, PathBuf}, str::FromStr};
use validator::{Validate, ValidationError};
use crate::{cli::*, paths::{self, Bak9Path}, Error, Result};

pub const CFG_BACKUP_STORAGE_DIR: &'static str = "backup_storage_dir";

macro_rules! select_config {
    ($cli:ident, $config:ident) => {
        match $config {
            Some(config) => config,
            None => &read_cli_config($cli)?
        }
    };
}

pub fn select_config_path(cli: &Cli) -> Result<PathBuf> {
    if let Some(config_path) = &cli.config_file {
        Ok(config_path.clone())
    } else {
        default_config_path()
    }
}

pub fn default_config_path() -> Result<PathBuf> {
    Ok(paths::home_dir()?
        .join(paths::HOME_CONFIG_DIR)
        .join(paths::BAK9_CONFIG_FILENAME))
}

pub(crate) fn read_cli_config(cli: &Cli) -> Result<BackupConfig> {
    match cli.config_file.as_ref() {
        Some(path) => read_config(Some(path.as_path())),
        None => read_config(None)
    }
}
        
    

pub fn read_config(config_path: Option<&Path>) -> Result<BackupConfig> {
    let config_path = if let Some(config_path) = config_path {
        if !config_path.exists() {
            return Err(Error::ConfigFileNotFound { path: config_path.to_str().unwrap().to_string() })
        } else {
            PathBuf::from(config_path)
        }
    } else {
        let default_config_path = default_config_path()?;
        if !default_config_path.exists() {
            return Err(Error::DefaultConfigFileNotFound { path: default_config_path.to_str().unwrap().to_string() })
        } else {
            default_config_path
        }
    };

    BackupConfig::read(&config_path)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
#[validate(schema(function = "BackupConfig::validate_schema"))]
#[validate]
pub struct BackupConfig {
    pub backup_storage_dir: String,

    #[serde(alias = "schedule", default = "Vec::new")]
    #[validate(nested)]
    pub schedules: Vec<BackupConfigSchedule>,

    #[serde(alias = "backup")]
    #[validate(nested)]
    pub backups: Vec<BackupConfigBackup>,

    #[serde(alias = "remote", default = "Vec::new")]
    #[validate(nested)]
    pub remotes: Vec<BackupConfigRemote>,

    #[serde(alias = "remote_group", default = "Vec::new")]
    #[validate(nested)]
    pub remote_groups: Vec<BackupConfigRemoteGroup>,
}

impl FromStr for BackupConfig {
    type Err = Error;

    fn from_str(config_content: &str) -> Result<Self> {
        let config: Self = toml::from_str(config_content)
            .map_err(|e| Error::ConfigParse { cause: e.to_string() })?;

        config.validate()
            .map_err(|e| Error::ConfigParse { cause: e.to_string() })?;

        Ok(config)
    }
}

impl BackupConfig {
    pub fn read(filepath: &Path) -> Result<Self> {
        let content = fs::read_to_string(filepath)
            .map_err(|e| Error::config_file(filepath, e))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| Error::config_file(filepath, e))?;

        config.validate()
            .map_err(|e| Error::config_file(filepath, e))?;

        Ok(config)
    }

    pub fn read_home() -> Result<Self> {
        let config_filepath = paths::home_dir()?
            .join(paths::HOME_CONFIG_DIR)
            .join(paths::BAK9_CONFIG_FILENAME);
        Self::read(&config_filepath)
    }

    pub fn backup_storage_dir_path(&self) -> PathBuf {
        paths::expand_path(&self.backup_storage_dir)
    }

    pub fn bak9_storage_dir(&self) -> Bak9Path {
        Bak9Path::StorageDir(self.backup_storage_dir_path())
    }

    pub fn remote<'cfg>(&'cfg self, name: &str) -> Result<&'cfg BackupConfigRemote> {
        self.remotes.iter()
            .find(|r| r.name == name)
            .ok_or_else(|| Error::ConfigReferenceNotFound { schema: "[remote]", name: name.to_string() })
    }

    pub fn remote_group<'cfg>(&'cfg self, name: &str) -> Result<&'cfg BackupConfigRemoteGroup> {
        self.remote_groups.iter()
            .find(|r| r.name == name)
            .ok_or_else(|| Error::ConfigReferenceNotFound { schema: "[remote_group]", name: name.to_string() })
    }

    pub fn schedule<'cfg>(&'cfg self, schedule_name: &str) -> Result<&'cfg BackupConfigSchedule> {
        if let Some(schedule) = self.schedules.iter().find(|s| s.name == schedule_name) {
            return Ok(schedule);
        }

        DEFAULT_SCHEDULES.iter()
            .find(|s| s.name == schedule_name)
           .ok_or_else(|| Error::ConfigReferenceNotFound { schema: "[schedule]", name: schedule_name.to_string() })
    }

    pub fn backup<'cfg>(&'cfg self, backup_name: &str) -> Result<&'cfg BackupConfigBackup> {
        self.backups.iter()
            .find(|b| b.name == backup_name)
            .ok_or_else(|| Error::ConfigReferenceNotFound { schema: "[backup]", name: backup_name.to_string() })
    }

    pub fn validate_schema(&self) -> std::result::Result<(), validator::ValidationError> {
        // ensure that all remote groups are referencing existing remotes
        for remote_group in &self.remote_groups {
            for group_remote_ref in &remote_group.remotes {
                self.remotes.iter()
                    .find(|r| &r.name == group_remote_ref)
                    .ok_or_else(|| ValidationError::new("").with_message(format!(
                        "remote_group['{}'].remote['{group_remote_ref}']: Unknown remote `{group_remote_ref}`",
                        remote_group.name).into()))?;
            }
        }
        Ok(())
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
    #[validate(does_not_contain(pattern = "__"))]
    pub name: String,
    pub source_dir: String,
    pub full_schedule: String,
    pub incremental_schedule: String,
    pub max_full: u32,

    #[serde(alias = "archive")]
    #[validate(nested)]
    pub archives: Vec<BackupConfigArchive>,

    #[serde(alias = "sync", default = "Vec::new")]
    #[validate(nested)]
    pub syncs: Vec<BackupConfigSync>
}

impl BackupConfigBackup {
    pub fn source_dir_path(&self) -> PathBuf {
        paths::expand_path(&self.source_dir)
    }

    pub fn archive<'cfg>(&'cfg self, schedule_name: &str) -> Result<&'cfg BackupConfigArchive> {
        self.archives.iter()
            .find(|b| b.schedule == schedule_name)
            .ok_or_else(|| Error::ConfigReferenceNotFound {
                schema: "Archive",
                name: format!("{}:{}", self.name.to_string(), schedule_name.to_string())
            })
    }
}
 
#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct BackupConfigArchive {
    pub schedule: String,
    pub max_archives: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct BackupConfigRemote {
    pub name: String,
    pub host: String,
    pub user: Option<String>,
    pub backup_storage_dir: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct BackupConfigRemoteGroup {
    pub name: String,
    pub remotes: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
#[validate(schema(function = "BackupConfigSync::validate_schema"))]
pub struct BackupConfigSync {
    pub remote: Option<String>,
    pub remote_group: Option<String>,
    pub sync_full: bool,
    pub sync_incremental: bool,
    pub sync_archive: bool,
}

impl BackupConfigSync {
    pub fn remotes<'cfg>(&self, config: &'cfg BackupConfig) -> Vec<&'cfg BackupConfigRemote> {
        if let Some(remote) = &self.remote {
            vec![config.remote(remote).unwrap()]
        } else if let Some(remote_group) = &self.remote_group {
            config.remote_group(remote_group).unwrap().remotes.iter()
                .map(|group_name| config.remote_group(group_name).unwrap())
                .flat_map(|group| &group.remotes)
                .map(|remote_name| config.remote(remote_name).unwrap())
                .collect()
        } else {
            unreachable!()
        }
    }

    fn validate_schema(&self) -> std::result::Result<(), validator::ValidationError> {
        if self.remote.is_none() && self.remote_group.is_none() {
            Err(ValidationError::new("either `remote` or `remote_group` must be set"))
        } else if self.remote.is_some() && self.remote_group.is_some() {
            Err(ValidationError::new("`remote` and `remote_group` are mutually exclusive"))
        } else {
            Ok(())
        }
    }
}

lazy_static::lazy_static!{
    static ref DEFAULT_SCHEDULES: Vec<BackupConfigSchedule> = vec![
        BackupConfigSchedule {
            name: "daily".to_string(),
            minute: Some(0),
            minutes: None,
            hour: Some(0),
            hours: None,
            day_of_week: None,
            days_of_week: None,
            day_of_month: None,
            days_of_month: None,
            month: None,
            months: None,
        },
        BackupConfigSchedule {
            name: "weekly".to_string(),
            minute: Some(0),
            minutes: None,
            hour: Some(0),
            hours: None,
            day_of_week: Some(DayOfWeek::Sunday),
            days_of_week: None,
            day_of_month: None,
            days_of_month: None,
            month: None,
            months: None,
        },
        BackupConfigSchedule {
            name: "monthly".to_string(),
            minute: Some(0),
            minutes: None,
            hour: Some(0),
            hours: None,
            day_of_week: None,
            days_of_week: None,
            day_of_month: Some(1),
            days_of_month: None,
            month: None,
            months: None,
        },
        BackupConfigSchedule {
            name: "quarterly".to_string(),
            minute: Some(0),
            minutes: None,
            hour: Some(0),
            hours: None,
            day_of_week: None,
            days_of_week: None,
            day_of_month: Some(1),
            days_of_month: None,
            month: None,
            months: Some(vec![1, 4, 7, 10]),
        },
        BackupConfigSchedule {
            name: "annual".to_string(),
            minute: Some(0),
            minutes: None,
            hour: Some(0),
            hours: None,
            day_of_week: None,
            days_of_week: None,
            day_of_month: Some(1),
            days_of_month: None,
            month: Some(1),
            months: None,
        }
    ];
}

pub const CONFIG_DEFAULTS: &'static str =
r#"backup_storage_dir = "/storage/backup"

[[backup]]
name = "home"
source_dir = "$HOME"
full_schedule = "monthly"
incremental_schedule = "daily"
max_full = 4

[[backup.archive]]
schedule = "quarterly"
max_archives = 5

[[backup.archive]]
schedule = "annual"
max_archives = 4
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn append_cfgs(cfgs: &[&str]) -> String {
        let mut config = CONFIG_DEFAULTS.to_string();
        for cfg in cfgs {
            config.push_str(cfg);
        }

        config
    }

    fn append_cfg(cfg: &str) -> String {
        format!("{}{}", CONFIG_DEFAULTS, cfg)
    }

    #[test]
    fn test_parse_default_config() {
        let config = BackupConfig::from_str(CONFIG_DEFAULTS).unwrap();
        assert_eq!("/storage/backup", config.backup_storage_dir);
        assert_eq!(0, config.schedules.len());
        assert_eq!(1, config.backups.len());
    }

    #[test]
    fn test_parse_sample_config() {
        let config = BackupConfig::from_str(CONFIG_SAMPLE).unwrap();
        assert_eq!(config.backup_storage_dir, "/storage/backup");
        assert_eq!("/storage/backup", config.backup_storage_dir);
        assert_eq!(0, config.schedules.len());
        assert_eq!(1, config.backups.len());
        assert_eq!(3, config.remotes.len());
        assert_eq!(1, config.remote_groups.len());
        assert_eq!(2, config.backups[0].syncs.len());
        assert_eq!("desktop1", config.remotes[1].name);
        assert_eq!("roam", config.remote_groups[0].name);
        assert_eq!(config.remotes[1].name, config.remote_groups[0].remotes[1]);
        assert_eq!(config.backups[0].syncs[0].remote_group, Some(config.remote_groups[0].name.to_string()));
    }

    #[test]
    fn test_default_schedule() {
        let config = BackupConfig::from_str(CONFIG_DEFAULTS).unwrap();
        assert_eq!("monthly", config.schedule("monthly").unwrap().name);
        assert_eq!(1, config.schedule("monthly").unwrap().day_of_month.unwrap());

        // override "monthly"
        let config = BackupConfig::from_str(&append_cfg(
            r#"[[schedule]]
            name = "monthly"
            minute = 0
            hour = 0
            day_of_month = 2
            "#)).unwrap();
        assert_eq!("monthly", config.schedule("monthly").unwrap().name);
        assert_eq!(2, config.schedule("monthly").unwrap().day_of_month.unwrap());
    }

    const CFG_SAMPLE_REMOTES: &'static str = 
        r#"[[remote]]
        name = "laptop1"
        host = "laptop1.local"
        backup_storage_dir = "/storage/backup"

        [[remote]]
        name = "desktop1"
        host = "desktop.local"
        backup_storage_dir = "/storage/backup"

        [[remote]]
        name = "cloud"
        host = "cloud.local"
        user = "backup"
        backup_storage_dir = "/home/$REMOTE_USER/backup"
        "#;

    #[test]
    fn test_parse_config_errors() {
        let config = append_cfgs(&[CFG_SAMPLE_REMOTES,
            r#"[[remote_group]]
            name = "bad"
            remotes = [
                "fail",
                "desktop1"
            ]
            "#]);
        assert_eq!(&BackupConfig::from_str(&config).unwrap_err().to_string(),
            "Config parsing error :: __all__: remote_group['bad'].remote['fail']: Unknown remote `fail`");
    }

    const CONFIG_SAMPLE: &'static str =
        r#"backup_storage_dir = "/storage/backup"

        [[backup]]
        name = "home"
        source_dir = "$HOME"
        full_schedule = "monthly"
        incremental_schedule = "daily"
        max_full = 4

        [[backup.archive]]
        schedule = "quarterly"
        max_archives = 5

        [[backup.archive]]
        schedule = "annual"
        max_archives = 4

        [[backup.sync]]
        remote_group = "roam"
        sync_full = true
        sync_incremental = true
        sync_archive = true

        [[backup.sync]]
        remote = "cloud"
        sync_full = true
        sync_incremental = true
        sync_archive = true

        [[remote]]
        name = "laptop1"
        host = "laptop1.local"
        backup_storage_dir = "/storage/backup"

        [[remote]]
        name = "desktop1"
        host = "desktop.local"
        backup_storage_dir = "/storage/backup"

        [[remote]]
        name = "cloud"
        host = "cloud.local"
        user = "backup"
        backup_storage_dir = "/home/$REMOTE_USER/backup"

        [[remote_group]]
        name = "roam"
        remotes = [
            "laptop1",
            "desktop1"
        ]
        "#;
}