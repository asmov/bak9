use std::{fs, path::{PathBuf, Path}};
use crate::{backup::{BackupRunName, BackupType}, config, error::{Error, Result}};

pub const HOME_CONFIG_DIR: &'static str = ".config/bak9";
pub const BAK9_CONFIG_FILENAME: &'static str = "bak9.toml";

pub const BACKUP_ARCHIVE_DIRNAME: &'static str = "archive";
pub const BACKUP_FULL_DIRNAME: &'static str = "full";
pub const BACKUP_INCREMENTAL_DIRNAME: &'static str = "incremental";
pub const BACKUP_LOGS_DIRNAME: &'static str = "logs";

pub const ENV_BAK9_HOME: &'static str = "BAK9_HOME";

pub const TAR_XZ_EXTENSION: &'static str = "tar.xz";

pub enum Bak9Path {
    StorageDir(PathBuf),
    FullBackup{ storage_dir: PathBuf, run_name: BackupRunName, path: PathBuf},
    IncrementalBackup{ storage_dir: PathBuf, run_name: BackupRunName, path: PathBuf },
    Archive{ storage_dir: PathBuf, run_name: BackupRunName, path: PathBuf },
    Log{ storage_dir: PathBuf, run_name: BackupRunName, path: PathBuf },
    UserConfig{ home_dir: PathBuf, path: PathBuf},
}

impl AsRef<Path> for Bak9Path {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl Bak9Path {
    pub fn storage_dir<P: AsRef<Path>>(storage_dir: P) -> Self {
        Self::StorageDir(storage_dir.as_ref().to_path_buf())
    }

    pub fn full_backup<P: AsRef<Path>>(storage_dir: P, run_name: &BackupRunName) -> Self {
        Self::FullBackup {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            path: storage_dir.as_ref().join(BACKUP_FULL_DIRNAME).join(&run_name),
            run_name: run_name.clone()
        }
    }

    pub fn incremental_backup<P: AsRef<Path>>(storage_dir: P, run_name: &BackupRunName) -> Self {
        Self::IncrementalBackup {
            storage_dir: storage_dir.as_ref().to_path_buf(), 
            path: storage_dir.as_ref().join(BACKUP_INCREMENTAL_DIRNAME).join(run_name),
            run_name: run_name.clone()
        }
    }

    pub fn backup<P: AsRef<Path>>(storage_dir: P, backup_type: BackupType, run_name: &BackupRunName) -> Self {
        match backup_type {
            BackupType::Full => Self::full_backup(storage_dir, run_name),
            BackupType::Incremental => Self::incremental_backup(storage_dir, run_name),
        }
    }

    pub fn archive<P: AsRef<Path>>(storage_dir: P, run_name: &BackupRunName) -> Self {
        Self::Archive {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            path: storage_dir.as_ref().join(BACKUP_ARCHIVE_DIRNAME).join(&run_name),
            run_name: run_name.clone()
        }
    }

    pub fn log<P: AsRef<Path>>(storage_dir: P, run_name: &BackupRunName) -> Self {
        Self::Log {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            path: storage_dir.as_ref().join(BACKUP_LOGS_DIRNAME).join(&run_name),
            run_name: run_name.clone()
        }
    }

    pub fn user_config<P: AsRef<Path>>(home_dir: P) -> Self {
        Self::UserConfig {
            home_dir: home_dir.as_ref().to_path_buf(),
            path: home_dir.as_ref().join(HOME_CONFIG_DIR).join(BAK9_CONFIG_FILENAME)
        }
    }

    pub fn as_path(&self) -> &Path {
        match self {
            Self::StorageDir(path) => path,
            Self::FullBackup{path, ..} => path,
            Self::IncrementalBackup{path, ..} => path,
            Self::Archive{path, ..} => path,
            Self::Log{path, ..} => path,
            Self::UserConfig{path, ..} => path,
        }
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.as_path().to_path_buf()
    }
}

impl From<Bak9Path> for PathBuf {
    fn from(value: Bak9Path) -> Self {
        value.to_path_buf()
    }
}

pub fn expand_path(path_str: &str) -> PathBuf {
    shellexpand::env(path_str).unwrap().to_string().into()
}
 
pub fn home_dir() -> Result<PathBuf> {
    let home: PathBuf = if let Ok(bak9_home) = std::env::var(ENV_BAK9_HOME) {
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
            .map_err(|e| Error::file_io(&home_config_dir, e))?;
    }

    if !home_config_file.exists() || force {
        fs::write(&home_config_file, config::CONFIG_DEFAULTS)
            .map_err(|e| Error::file_io(&home_config_file, e))?;
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
            .map_err(|e| Error::file_io(&backup_storage_dir, e))?;
    }

    let backup_storage_dir = backup_storage_dir.canonicalize()
        .map_err(|e| Error::file_io(&backup_storage_dir, e))?;

    for subdir in backup_storage_subdirs(&backup_storage_dir) {
        fs::create_dir_all(&subdir)
            .map_err(|e| Error::file_io(&subdir, e))?;
    }

    Ok(())
}

pub fn verify_backup_dirs(config: &config::BackupConfig) -> Result<()> {
    let backup_storage_dir = &config.backup_storage_dir_path();
    let backup_storage_dir = backup_storage_dir.canonicalize()
        .map_err(|e| Error::configured_dir(&backup_storage_dir, config::CFG_BACKUP_STORAGE_DIR, e))?;

    for subdir in backup_storage_subdirs(&backup_storage_dir) {
        subdir.canonicalize()
            .map_err(|e| Error::configured_subdir(&subdir, config::CFG_BACKUP_STORAGE_DIR, e))?;
    }

    Ok(())
}

