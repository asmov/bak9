use std::path::Path;
use colored::Colorize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config file error: {path} :: {cause}")]
    ConfigFile { path: String, cause: String },

    #[error("Config file {} not found. Have you ran {} yet?", path.cyan(), "bak9 config".yellow())]
    DefaultConfigFileNotFound { path: String },

    #[error("Config file {} not found.", path.cyan())]
    ConfigFileNotFound { path: String },

    #[error("File IO error: {path} :> {cause}")]
    FileIO{ path: String, cause: String },

    #[error("Config item {} not found for schema {}", name.cyan(), schema.cyan())]
    ConfigReferenceNotFound { schema: &'static str, name: String },

    #[error("Directory {} not found. (config: {})", path.cyan(), config_key.cyan())]
    ConfiguredDirNotFound { path: String, config_key: String },

    #[error("Subdirectory {} not found. (config: {})", path.cyan(), config_key.cyan())]
    ConfiguredSubdirNotFound { path: String, config_key: String },

    #[error("Failed to {}: {cause}", "rsync".yellow())]
    RsyncError { cause: String },

    #[error("{0}")]
    Generic(String)
}

impl Error {
    pub fn config_file(path: &Path, err: impl std::error::Error) -> Self {
        Self::ConfigFile {
            path: path.to_str().unwrap().to_string(),
            cause: err.to_string()
        }
    }

    pub fn configured_dir(path: &Path, config_key: &str, _e: std::io::Error) -> Self {
        Self::ConfiguredDirNotFound {
            config_key: config_key.to_string(),
            path: path.to_str().unwrap().to_string(),
        }
    }

    pub fn configured_subdir(path: &Path, config_key: &str, _e: std::io::Error) -> Self {
        Self::ConfiguredSubdirNotFound {
            config_key: config_key.to_string(),
            path: path.to_str().unwrap().to_string(),
        }
    }

    pub fn file_io(path: &Path, err: impl std::error::Error) -> Self {
        Self::FileIO {
            path: path.to_str().unwrap().to_string(),
            cause: err.to_string()
        }
    }

    pub fn rsync(output: std::process::Output) -> Self {
        Self::RsyncError {
            cause: String::from_utf8(output.stderr).unwrap()
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
