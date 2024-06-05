use std::path::Path;
use colored::Colorize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config file error: {path} :> {cause}")]
    ConfigFile { path: String, cause: String },

    #[error("Config file {path} not found. Have you ran `{}` yet?", "bak9 setup".yellow())]
    ConfigFileNotFound { path: String },

    #[error("File IO error: {path} :> {cause}")]
    FileIO{ path: String, cause: String },

    #[error("Path {path} not accessible. Specified by config {config_key} :: {cause}")]
    ConfiguredPathNotAccessible{ config_key: String, path: String, cause: String },

    #[error("Config {schema} not found: {identifier}")]
    ConfigNotFound { schema: &'static str, identifier: String },
}

impl Error {
    pub fn new_config_file(path: &Path, err: impl std::error::Error) -> Self {
        Self::ConfigFile {
            path: path.to_str().unwrap().to_string(),
            cause: err.to_string()
        }
    }

    pub fn new_config(path: &Path, config: impl std::fmt::Display, msg: &str) -> Self {
        Self::ConfigFile {
            path: path.to_str().unwrap().to_string(),
            cause: format!("{msg} :: {config}")
        }
    }

    pub fn new_configured_path(path: &Path, config_key: &str, e: std::io::Error) -> Self {
        Self::ConfiguredPathNotAccessible {
            config_key: config_key.cyan().to_string(),
            path: path.to_str().unwrap().cyan().to_string(),
            cause: e.to_string()
        }
    }

    pub fn new_file_io(path: &Path, err: impl std::error::Error) -> Self {
        Self::FileIO {
            path: path.to_str().unwrap().to_string(),
            cause: err.to_string()
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

