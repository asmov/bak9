use std::{fs, path::{PathBuf, Path}};
use crate::{config, error::{Error, Result}};

pub const HOME_CONFIG_DIR: &'static str = ".config/bak9";
pub const BAK9_CONFIG_FILENAME: &'static str = "bak9.toml";

pub const BACKUP_ARCHIVE_DIRNAME: &'static str = "archive";
pub const BACKUP_FULL_DIRNAME: &'static str = "full";
pub const BACKUP_INCREMENTAL_DIRNAME: &'static str = "incremental";
pub const BACKUP_LOGS_DIRNAME: &'static str = "logs";

pub const BAK9_HOME: &'static str = "BAK9_HOME";

pub fn expand_path(path_str: &str) -> PathBuf {
    shellexpand::env(path_str).unwrap().to_string().into()
}
 
pub fn home_dir() -> Result<PathBuf> {
    let home: PathBuf = if let Ok(bak9_home) = std::env::var(BAK9_HOME) {
        PathBuf::from(bak9_home)
    } else {
        option_env!("HOME")
            .ok_or_else(|| Error::FileIO { path: "$HOME".to_string(), cause: "$HOME is not set".to_string() })?
            .into()
    };

    Ok(home)
}

pub fn setup_home_config(force: bool) -> Result<()> {
    let home_config_dir = home_dir()?.join(HOME_CONFIG_DIR);
    let home_config_file = home_config_dir.join(BAK9_CONFIG_FILENAME);

    if !home_config_dir.exists() {
        fs::create_dir_all(&home_config_dir)
            .map_err(|e| Error::new_file_io(&home_config_dir, e))?;
    }

    if !home_config_file.exists() || force {
        fs::write(&home_config_file, config::CONFIG_DEFAULTS)
            .map_err(|e| Error::new_file_io(&home_config_file, e))?;
    }

    Ok(())
}

pub fn backup_storage_subdirs(backup_storage_dir: &Path) -> Vec<PathBuf> {
    vec![
        backup_storage_dir.join(BACKUP_ARCHIVE_DIRNAME),
        backup_storage_dir.join(BACKUP_FULL_DIRNAME),
        backup_storage_dir.join(BACKUP_INCREMENTAL_DIRNAME),
        backup_storage_dir.join(BACKUP_LOGS_DIRNAME)
    ]
}

pub fn setup_backup_storage_dir(backup_storage_dir: &Path) -> Result<()> {
    if !backup_storage_dir.exists() {
        fs::create_dir_all(&backup_storage_dir)
            .map_err(|e| Error::new_file_io(&backup_storage_dir, e))?;
    }

    let backup_storage_dir = backup_storage_dir.canonicalize()
        .map_err(|e| Error::new_file_io(&backup_storage_dir, e))?;

    for subdir in backup_storage_subdirs(&backup_storage_dir) {
        fs::create_dir_all(&subdir)
            .map_err(|e| Error::new_file_io(&subdir, e))?;
    }

    Ok(())
}

pub fn verify_backup_dirs(config: &config::BackupConfig) -> Result<()> {
    let backup_storage_dir = &config.backup_storage_dir_path();
    let backup_storage_dir = backup_storage_dir.canonicalize()
        .map_err(|e| Error::new_configured_path(&backup_storage_dir, config::KEY_BACKUP_STORAGE_DIR, e))?;

    for subdir in backup_storage_subdirs(&backup_storage_dir) {
        subdir.canonicalize()
            .map_err(|e| Error::new_configured_path(&subdir, config::KEY_BACKUP_STORAGE_DIR, e))?;
    }

    Ok(())
}

