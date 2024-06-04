use std::{fs, path::{PathBuf, Path}};
use crate::{config, error::{Error, Result}};

pub const HOME_CONFIG_DIR: &'static str = ".config/bak9/backup";
pub const CONFIG_FILENAME: &'static str = "config.toml";

pub const BACKUP_ARCHIVE_DIRNAME: &'static str = "archive";
pub const BACKUP_FULL_DIRNAME: &'static str = "full";
pub const BACKUP_INCREMENTAL_DIRNAME: &'static str = "incremental";
pub const BACKUP_LOGS_DIRNAME: &'static str = "logs";

#[cfg(test)]
use crate::testing;

pub fn config_path(path_str: &str) -> PathBuf {
    #[cfg(not(test))]
    return PathBuf::from(&path_str);

    #[cfg(test)] {
        let path_str = path_str
            .replace(testing::TEST_VAR_CARGO_MANIFEST_DIR, env!("CARGO_MANIFEST_DIR"))
            .replace(testing::TEST_VAR_CARGO_TARGET_TMPDIR, option_env!("CARGO_TARGET_TMPDIR").unwrap());
        PathBuf::from(&path_str)
    }
}
 
pub fn home_dir() -> Result<PathBuf> {
    #[cfg(not(test))] {
        let home = option_env!("HOME")
            .ok_or_else(|| Error::FileIO { path: "$HOME".to_string(), cause: "$HOME is not set".to_string() })?;
        return Ok(PathBuf::from(home));
    }

    #[cfg(test)] {
        let home = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(testing::TEST_HOME_DIR)
            .to_str().unwrap()
            .to_string();
        return Ok(PathBuf::from(home))
    }
}

pub fn setup_home_config(force: bool) -> Result<()> {
    let home_config_dir = home_dir()?.join(HOME_CONFIG_DIR);
    let home_config_file = home_config_dir.join(CONFIG_FILENAME);

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

pub fn setup_backup_dirs(config: &config::BackupConfig, force: bool) -> Result<()> {
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

