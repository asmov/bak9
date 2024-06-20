use std::path::Path;
use colored::Colorize;
use chrono::Timelike;
use crate::strings;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config file error: {path} :> {cause}")]
    ConfigFile { path: String, cause: String },

    #[error("Config file {} not found. Have you ran {} yet?", path.cyan(), "bak9 config".yellow())]
    DefaultConfigFileNotFound { path: String },

    #[error("Config file {} not found.", path.cyan())]
    ConfigFileNotFound { path: String },

    #[error("File IO error: {path} :> {cause}")]
    FileIO{ path: String, cause: String },

    #[error("Path {} specified by config key {} is not accessible :: {cause}", path.cyan(), config_key.cyan())]
    ConfiguredPathNotAccessible{ config_key: String, path: String, cause: String },

    #[error("Config item {} not found for schema {}", name.cyan(), schema.cyan())]
    ConfigReferenceNotFound { schema: &'static str, name: String },

    #[error("Directory {} not found. (config: {})", path.cyan(), config_key.cyan())]
    ConfiguredDirNotFound { path: String, config_key: String },

    #[error("Subdirectory {} not found. (config: {})", path.cyan(), config_key.cyan())]
    ConfiguredSubdirNotFound { path: String, config_key: String },

    #[error("{0}")]
    Generic(String)
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
            config_key: config_key.to_string(),
            path: path.to_str().unwrap().to_string(),
            cause: e.to_string()
        }
    }

    pub fn new_configured_dir(path: &Path, config_key: &str, _e: std::io::Error) -> Self {
        Self::ConfiguredDirNotFound {
            config_key: config_key.to_string(),
            path: path.to_str().unwrap().to_string(),
        }
    }

    pub fn new_configured_subdir(path: &Path, config_key: &str, _e: std::io::Error) -> Self {
        Self::ConfiguredSubdirNotFound {
            config_key: config_key.to_string(),
            path: path.to_str().unwrap().to_string(),
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

pub fn make_log_prefix(topic: &str, status: &str, color: colored::Color) -> String {
    let now = chrono::Local::now();
    let prefix = format!("[{:0>2}:{:0>2}:{:0>2} {topic}]{status}",
        now.hour(),
        now.minute(),
        now.second());

    prefix.color(color).to_string()
}

pub fn bak9_error_log_prefix() -> String {
    make_log_prefix(strings::BAK9, " error:", colored::Color::Red)
}

pub fn bak9_info_log_prefix() -> String {
    make_log_prefix(strings::BAK9, "", colored::Color::Green)
}

