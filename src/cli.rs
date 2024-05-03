use std::path::PathBuf;
use clap::Parser;

use crate::PathExt;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(value_parser = validate_file)]
    pub file: PathBuf,

    #[arg(value_parser = validate_dir, help = "If not specified, the backup is created in the same directory as FILE.")]
    pub dir: Option<PathBuf>,

    #[arg(short, default_value_t = false, help = "Delete all backups of FILE")]
    pub delete: bool,

    #[arg(short, value_parser = clap::value_parser!(u8).range(1..),
        default_value_t = 10, help = "Number of backups to keep before pruning")]
    pub num: u8,
}

impl Cli {
    pub fn dir(&self) -> PathBuf {
        match &self.dir {
            Some(dir) => dir.clone(),
            None => self.file.parent().expect("Expected parent directory").to_path_buf().clone(),
        }
    }
}

fn validate_path(path: &str, filetype: &'static str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path)
        .canonicalize()
        .map_err(|_| format!("{filetype} not found: {:?}", path))?;

    if !path.exists() {
        return Err(format!("{filetype} not found: {:?}", path))
    }
    
    Ok(path)
}

fn validate_file(path: &str) -> Result<PathBuf, String> {
    let path = validate_path(path, "File")?;
    if !path.is_file() {
        Err(format!("Source path is not a file: {:?}", path))
    } else if path.filename_str().is_none() {
        return Err(format!("Invalid source file: {:?}", path))
    } else {
        Ok(path)
    }
}

fn validate_dir(path: &str) -> Result<PathBuf, String> {
    let path = validate_path(path, "Directory")?;
    if !path.is_dir() {
        return Err(format!("Destination path is not a directory: {:?}", path))
    } else {
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_cli() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert()
    }
}