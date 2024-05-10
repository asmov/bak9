use std::path::PathBuf;
use clap::{Parser, Subcommand};

use crate::{PathExt, E_STR};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Command>,

    #[arg(value_parser = validate_file)]
    pub file: PathBuf,

    #[arg(value_parser = validate_dir, help = "[default: Same directory as FILE. '-': The user's app data directory]")]
    pub dir: Option<PathBuf>,

    #[arg(short, value_parser = clap::value_parser!(u8).range(1..),
        default_value_t = 10, help = "Number of backups to keep before pruning")]
    pub num: u8,

    #[arg(short, help = "Force the operation without confirmation")]
    pub force: bool,

    #[arg(short, help = "Suppress all output")]
    pub quiet: bool
}


#[derive(Subcommand)]
pub enum Command {
    #[command(name = "ls", about = "List all backups of FILE in DIR")]
    List,
    #[command(name = "rm", about = "Deletes all backups of FILE in DIR")]
    Wipe, 
    #[command(name = "diff", about = "Shows the differences between FILE and BAK.N")]
    Diff {
        #[arg(default_value_t = 0, help = "The BAK index to compare FILE with")]
        index: u8,
    }
}

impl Cli {
    pub fn dir(&self) -> PathBuf {
        match &self.dir {
            Some(dir) => {
                // handle passing Cli parameters manually
                if dir.to_str().expect(E_STR) == "-" {
                    crate::os::user_app_data_dir(true, crate::BAK9.into())
                        .expect("Failed to get user app data directory")
                } else {
                    dir.clone()
                }
            },
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
    let path = if path == "-" {
        crate::os::user_app_data_dir(true, crate::BAK9.into())
            .map_err(|e| e.to_string())?
    } else {
        validate_path(path, "Directory")?
    };

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