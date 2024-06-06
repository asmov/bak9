use std::{fs, path::{PathBuf, Path}};
use validator::{Validate, ValidationError};
use crate::{cli, error::{Error, Result}, paths};

pub const KEY_BACKUP_STORAGE_DIR: &'static str = "backup_storage_dir";

pub fn select_config_path(cli: &cli::Cli) -> Result<PathBuf> {
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

pub fn read_config(config_path: Option<&PathBuf>) -> Result<BackupConfig> {
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
        let config_filepath = paths::home_dir()?
            .join(paths::HOME_CONFIG_DIR)
            .join(paths::BAK9_CONFIG_FILENAME);
        Self::read(&config_filepath)
    }

    pub fn backup_storage_dir_path(&self) -> PathBuf {
        paths::expand_path(&self.backup_storage_dir)
    }

    pub fn schedule<'cfg>(&'cfg self, schedule_name: &str) -> Result<&'cfg BackupConfigSchedule> {
        self.schedules.iter()
            .find(|s| s.name == schedule_name)
            .ok_or_else(|| Error::ConfigReferenceNotFound { schema: "[schedule]", name: schedule_name.to_string() })
    }

    pub fn backup<'cfg>(&'cfg self, backup_name: &str) -> Result<&'cfg BackupConfigBackup> {
        self.backups.iter()
            .find(|b| b.name == backup_name)
            .ok_or_else(|| Error::ConfigReferenceNotFound { schema: "[backup]", name: backup_name.to_string() })
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

pub const CONFIG_DEFAULTS: &'static str = r#"
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
max_archives = 4

[[backup.archive]]
schedule = "annual"
max_archives = 3
"#;

