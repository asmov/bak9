//! # bak9  
//! 
//! Creates a backup `.bak` copy of **FILE**.
//! 
//! Usage: `bak [OPTION]... FILE [DIR]`
//! 
//! If **DIR** is not specified, the copy is created in the same directory as FILE.
//! 
//! If *multiple* backups of FILE exist, the filename extension used will be: `.bak.N`.
//! 
//! With multiple backups, the most recent backup will be always `bak.0`. Previous
//! copies will have their filename extension shifted by 1 (e.g., `bak.1` -> `bak.2`).
//! 
//! Pruning (deletion) occurs after `-n NUM` backups. 
//! 
//! If the current backup is no *diff*erent than its predecessor, copying will be skipped. 
//! 
//! # Options
//! 
//! - `-d`  
//! Deletes all backup files for the source FILE.
//! 
//! - `-n NUM`  
//! Creates at most **NUM** backup files.  
//! If not specified, defaults to 10 (0-9).

pub mod cli;

use std::{fs, path::{Path, PathBuf}};
use clap::Parser;
use file_diff;
use thiserror;
use colored::Colorize;

const BAK: &str = "bak";
const BAK_DOT: &str = "bak.";
const BAK_0: &str = "bak.0";
const BAK_1: &str = "bak.1";

const E_STR: &str = "Expected string";
const E_FILENAME: &str = "Expected filename";

/// Ergonomic methods for working with paths
trait PathExt {
    fn append_extension(self, ext: &str) -> PathBuf;
    fn filename_string(self) -> Option<String>;
    fn filename_str<'s>(&'s self) -> Option<&'s str>;
}

impl PathExt for PathBuf {
    fn append_extension(self, ext: &str) -> PathBuf {
        self.with_extension(format!("{}.{}", self.extension().unwrap_or_default().to_str().expect(E_STR), ext))
    }

    fn filename_string(self) -> Option<String> {
        match self.file_name() {
            Some(filename) => match filename.to_str() {
                Some(filename) => Some(filename.to_owned()),
                None => None
            },
            None => None
        }
    }

    fn filename_str<'s>(&'s self) -> Option<&'s str> {
        match self.file_name() {
            Some(filename) => filename.to_str(),
            None => None
        }
    }
}

impl PathExt for &Path {
    fn append_extension(self, ext: &str) -> PathBuf {
        self.with_extension(format!("{}.{}", self.extension().unwrap_or_default().to_str().expect(E_STR), ext))
    }

    fn filename_string(self) -> Option<String> {
        match self.file_name() {
            Some(filename) => match filename.to_str() {
                Some(filename) => Some(filename.to_owned()),
                None => None
            },
            None => None
        }
    }

    fn filename_str<'s>(&'s self) -> Option<&'s str> {
        match self.file_name() {
            Some(filename) => filename.to_str(),
            None => None
        }
    }
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum IoOp {
    Read,
    Write,
    Delete,
    Rename
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to {op} {path}: {cause}")]
    IO { op: IoOp, path: String, cause: String },

    #[error("Unable to copy {src} to {dest}: {cause}")]
    Copy { src: String, dest: String, cause: String },
}

impl Error {
    pub fn io(op: IoOp, path: PathBuf, cause: std::io::Error) -> Self {
        Self::IO { op, path: path.to_str().expect(E_STR).cyan().to_string(), cause: cause.to_string() }
    }

    pub fn copy(source: &Path, destination: &Path, cause: std::io::Error) -> Self {
        Self::Copy {
            src: source.to_str().expect(E_STR).cyan().to_string(),
            dest: destination.to_str().expect(E_STR).cyan().to_string(),
            cause: cause.to_string() }
    }
}

/// Entry point
pub fn run() -> std::process::ExitCode {
    match run_with(cli::Cli::parse()) {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{} {err}", "error:".red());
            std::process::ExitCode::FAILURE
        }
    }
}

pub fn run_with(cli: cli::Cli) -> Result<(), Error> {
    let result = if cli.delete {
        run_delete(&cli)
    } else {
        run_backup(&cli)
    };

    result
}

/// Performs a wipe of all `.bak` files in the directory.
fn run_delete(cli: &cli::Cli) -> Result<(), Error> {
    let bak_filepaths = list_bak_n_files(&cli.file, &cli.dir())?;

    for bak_filepath in bak_filepaths {
        std::fs::remove_file(&bak_filepath)
            .map_err(|e| Error::io(IoOp::Delete, bak_filepath, e))?;
    }

    // wipe the .bak if it exists as well
    let bak_filepath = cli.dir()
        .join(cli.file.filename_str().expect(E_FILENAME))
        .append_extension(BAK);

    if bak_filepath.exists() {
        std::fs::remove_file(&bak_filepath)
            .map_err(|e| Error::io(IoOp::Delete, bak_filepath, e))?;
    }

    Ok(())
}

/// Retrieves a list of all `.bak.N` files in the directory.
fn list_bak_n_files(file: &Path, dir: &Path, ) -> Result<Vec<PathBuf>, Error> {
    let bak_n_file_pattern = file
        .append_extension(BAK_DOT)
        .filename_string().expect(E_FILENAME);

    let dir = dir.read_dir()
        .map_err(|e| Error::io(IoOp::Read, dir.to_path_buf(), e))?;
    let paths = dir
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry.path()),
            Err(_) => None
        })
        .filter(|path| path.is_file())
        .filter(|filepath| {
            let filename = match filepath.file_name() {
                Some(filename) => filename.to_str().expect(E_STR),
                None => return false
            };

            if filename.starts_with(&bak_n_file_pattern) {
                matches!(filename.trim_start_matches(&bak_n_file_pattern).parse::<u32>(), Ok(_))
            } else {
                false
            }
        })
        .collect();

    Ok(paths)
}

/// Performs a copy
fn run_backup(cli: &cli::Cli) -> Result<(), Error> {
    let source_filename = cli.file.filename_str().expect(E_FILENAME);
    let last_bak = find_last_bak(&cli.file, &cli.dir());

    // check to see if a backup is necessary, using a file diff
    if let Some(last_bak_filepath) = &last_bak {
        let mut file = std::fs::File::open(&cli.file)
            .map_err(|e| Error::io(IoOp::Read, cli.file.clone(), e))?;
        let mut last_bak = std::fs::File::open(&last_bak_filepath)
            .map_err(|e| Error::io(IoOp::Read, last_bak_filepath.clone(), e))?;

        if file_diff::diff_files(&mut file, &mut last_bak) {
            return Ok(());
        }
    }

    let bak_filepath = if let Some(last_bak_filepath) = &last_bak {
        if cli.num == 1 {
            run_delete(cli)?;

            cli.dir()
                .join(&source_filename)
                .append_extension(BAK)
        } else if last_bak_filepath.extension().expect("Expected .bak") == BAK {
            shift_bak_files(&cli.file, &cli.dir(), cli.num)?;

            let bak1_filepath = cli.dir()
                .join(&source_filename)
                .append_extension(BAK_1);

            fs::rename(last_bak_filepath, &bak1_filepath)
                .map_err(|e| Error::io(IoOp::Rename, bak1_filepath, e))?;

            cli.dir()
                .join(&source_filename)
                .append_extension(BAK_0)
        } else {
            shift_bak_files(&cli.file, &cli.dir(), cli.num)?;

            cli.dir()
                .join(&source_filename)
                .append_extension(BAK_0)
        }
    } else { 
        cli.dir()
            .join(&source_filename)
            .append_extension(BAK)
    };

    fs::copy(&cli.file, &bak_filepath)
        .map_err(|e| Error::copy(&cli.file, &bak_filepath, e))?;

    Ok(())
}

/// Increments the filename extension of all `.bak.N` files in the directory.
fn shift_bak_files(file: &Path, dir: &Path, num: u8) -> Result<(), Error> {
    let mut bak_filepaths = list_bak_n_files(file, dir)?;

    // 0, 1, ..
    bak_filepaths.sort_by(|a, b| a.cmp(b));

    // prune all excess backups
    if bak_filepaths.len() >= num as usize {
        let prune_amount = bak_filepaths.len() - num as usize + 1;
        for _i in 0..prune_amount {
            let bak_filepath = bak_filepaths.pop().expect("Expected array value");
            std::fs::remove_file(&bak_filepath)
                .map_err(|e| Error::io(IoOp::Delete, bak_filepath, e))?;
        }
    }

    let bak_n_file_pattern = file
        .append_extension(BAK_DOT)
        .filename_string().expect(E_FILENAME);

    // shift each up by 1
    let source_filename = file.filename_string().expect(E_FILENAME);
    for bak_filepath in bak_filepaths.into_iter().rev() {
        let n = bak_filepath.filename_str().expect(E_FILENAME)
            .trim_start_matches(&bak_n_file_pattern)
            .parse::<u32>()
            .expect("Expected numeric extension");
        let bak_next_filepath = dir.join(&source_filename)
            .append_extension(BAK)
            .append_extension((n + 1).to_string().as_str());
        fs::rename(bak_filepath, &bak_next_filepath)
            .map_err(|e| Error::io(IoOp::Write, bak_next_filepath, e))?;
    }

    Ok(())
}

/// Returns either a `.bak` or `.bak.0` file if it exists.
fn find_last_bak(file: &Path, dir: &Path) -> Option<PathBuf> {
    let bak_file = dir.join(file.filename_string().expect(E_FILENAME))
        .append_extension(BAK);
    if bak_file.exists() {
        Some(bak_file)
    } else {
        let bak0_file = dir.join(file.filename_string().expect(E_FILENAME))
            .append_extension(BAK_0);
        if bak0_file.exists() {
            Some(bak0_file)
        } else {
            None
        }
    }
}
