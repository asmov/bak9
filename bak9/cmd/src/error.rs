use std::path::Path;

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

