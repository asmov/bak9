use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(value_parser = validate_file)]
    pub file: PathBuf,

    #[arg(value_parser = validate_dir, help = "If not specified, the backup is created in the same directory as FILE.")]
    pub dir: Option<PathBuf>,

    #[arg(short, default_value_t = false, help = "Delete all backups of FILE")]
    pub delete: bool,

    #[arg(short, default_value_t = 10, help = "Number of backups to keep before pruning")]
    pub num: u8,
}

impl Cli {
    pub fn dir(&self) -> PathBuf {
        match &self.dir {
            Some(dir) => dir.clone(),
            None => self.file.parent().unwrap().to_path_buf().clone(),
        }
    }
}

fn validate_file(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("File not found: {:?}", path))
    }
}

fn validate_dir(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Directory not found: {:?}", path))
    }
}