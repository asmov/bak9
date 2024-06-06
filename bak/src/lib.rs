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
pub mod os;

use std::{fs, io::Write, path::{Path, PathBuf}};
use clap::Parser;
use file_diff;
use thiserror;
use colored::Colorize;

pub const BAK9: &str = "bak9";
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
        if let Some(prev_ext) = self.extension() {
            self.with_extension(format!("{prev_ext}.{ext}", prev_ext = prev_ext.to_str().expect(E_STR)))
        } else {
            self.with_extension(ext)
        }
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
        if let Some(prev_ext) = self.extension() {
            self.with_extension(format!("{prev_ext}.{ext}", prev_ext = prev_ext.to_str().expect(E_STR)))
        } else {
            self.with_extension(ext)
        }
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
    Rename,
    Create
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to {op} {path}: {cause}")]
    IO { op: IoOp, path: String, cause: String },

    #[error("Unable to copy {src} to {dest}: {cause}")]
    Copy { src: String, dest: String, cause: String },

    #[error("Invalid index for {src}: {index}")]
    Index { src: String, index: u8 },

    #[error("{0}")]
    Generic(String)
}

impl Error {
    pub fn io(op: IoOp, path: &Path, cause: std::io::Error) -> Self {
        Self::IO { op, path: path.to_str().expect(E_STR).cyan().to_string(), cause: cause.to_string() }
    }

    pub fn index(source: &Path, index: u8) -> Self {
        Self::Index { src: source.to_str().expect(E_STR).cyan().to_string(), index }
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
    match cli.subcommand {
        None => run_backup(&cli),
        Some(cli::Command::List) => run_list(&cli),
        Some(cli::Command::Wipe) => run_wipe(&cli),
        Some(cli::Command::Diff { index }) => run_diff(&cli, index)
    }
}

fn remove_app_data_subdir(app_data_dir: &Path, app_data_subdir: &Path) -> Result<(), Error> {
    let mut subdir = app_data_subdir.to_path_buf();

    while subdir.is_dir() && subdir != app_data_dir && subdir.starts_with(app_data_dir) {
        let mut dir_contents = fs::read_dir(&subdir)
            .map_err(|e| Error::io(IoOp::Read, &subdir, e))?;

        if dir_contents.next().is_none() {
            fs::remove_dir(&subdir)
                .map_err(|e| Error::io(IoOp::Delete, &subdir, e))?;
        } else {
            break;
        }

        subdir.pop();
    }

    Ok(())
}

fn confirm_wipe(source_file: &Path, dir: &Path, force: bool) -> bool {
    if force {
        return true
    }

    let mut input = String::new();
    print!("{} Delete all backups of {} in {}? {} ",
        "confirm:".bright_yellow(), 
        source_file.filename_str().expect(E_FILENAME).cyan(),
        sanitize_path_str(dir.to_str().expect(E_STR)).cyan(),
        "[y/N]:".magenta());

    std::io::stdout().flush()
        .expect("Failed to flush stdout");
    std::io::stdin().read_line(&mut input)
        .expect("Failed to read input");

    input.trim()
        .to_lowercase() == "y"
}

fn run_wipe(cli: &cli::Cli) -> Result<(), Error> {
    let dir = cli.dir();
    let app_data_dir = os::user_app_data_dir(true, BAK9.into())
        .map_err(|e| Error::Generic(e.to_string()))?;
    
    if dir == app_data_dir {
        match mirror_dir(&app_data_dir, &cli.file, false) {
            Ok(mirror_dir) => {
                if confirm_wipe(&cli.file, &mirror_dir, cli.force) {
                    wipe(&cli.file, &mirror_dir)?;
                    remove_app_data_subdir(&app_data_dir, &mirror_dir)?;
                }
            },
            Err(_) => {}
        }
    } else {
        if confirm_wipe(&cli.file, &dir, cli.force) {
            wipe(&cli.file, &dir)?;
        }
    }

    // if dir was not specified (default), continue on to wipe the app data dir
    if cli.dir.is_some() {
        return Ok(())
    }

    if dir != app_data_dir {
        match mirror_dir(&app_data_dir, &cli.file, false) {
            Ok(mirror_dir) => {
                if confirm_wipe(&cli.file, &mirror_dir, cli.force) {
                    wipe(&cli.file, &mirror_dir)?;
                    remove_app_data_subdir(&app_data_dir, &mirror_dir)?;
                }
            },
            Err(_) => {}
        }
    }

    Ok(())
}

fn run_list(cli: &cli::Cli) -> Result<(), Error> {
    let dir = cli.dir();
    let app_data_dir = os::user_app_data_dir(true, BAK9.into())
        .map_err(|e| Error::Generic(e.to_string()))?;

    if dir != app_data_dir {
        print_list_backups(&cli.file, &dir)?;
    }

    match mirror_dir(&app_data_dir, &cli.file, false) {
        Ok(mirror_dir) => print_list_backups(&cli.file, &mirror_dir)?,
        Err(_) => {}
    }

    Ok(())
}

fn print_list_backups(source_file: &Path, dir: &Path) -> Result<(), Error> {
    if !dir.is_dir() {
        return Ok(())
    }

    let mut bak_filepaths = list_bak_n_files(source_file, dir)?;

    let bak_file = dir.join(source_file.filename_str().expect(E_FILENAME))
        .append_extension(BAK);

    if bak_file.exists() {
        bak_filepaths.push(bak_file);
        bak_filepaths.sort();
    }

    if bak_filepaths.is_empty() {
        return Ok(())
    }

    println!("Backups of {file} in {dir}:",
        file = sanitize_path_str(source_file.to_str().expect(E_STR)).cyan(),
        dir = sanitize_path_str(dir.to_str().expect(E_STR)).cyan());

    for bak_filepath in bak_filepaths {
        println!("    {}", bak_filepath.filename_str().expect(E_STR).green());
    }

    Ok(())
}

fn run_diff(cli: &cli::Cli, index: u8) -> Result<(), Error> {
    let source_file = &cli.file;
    let mut dir = cli.dir();
    let app_data_dir = os::user_app_data_dir(true, BAK9.into())
        .map_err(|e| Error::Generic(e.to_string()))?;

    if dir == app_data_dir {
        dir = mirror_dir(&app_data_dir, source_file, false)
            .map_err(|_| Error::index(source_file, index))?;
    }

    let bak_file = dir.join(source_file.filename_str().expect(E_FILENAME))
        .append_extension(BAK);

    if bak_file.exists() {
        if index > 0 {
            return Err(Error::index(source_file, index));
        }

        os::print_diff(source_file, &bak_file)
    } else {
        let bak_filepaths = list_bak_n_files(source_file, &dir)?;
        let bak_file = bak_filepaths.get(index as usize)
            .ok_or_else(|| Error::index(source_file, index))?;

        os::print_diff(source_file, bak_file)
    }
}

/// Performs a wipe of all `.bak` files in the directory.
fn wipe(source_file: &Path, dest_dir: &Path) -> Result<(), Error> {
    let bak_filepaths = list_bak_n_files(source_file, dest_dir)?;

    for bak_filepath in bak_filepaths {
        std::fs::remove_file(&bak_filepath)
            .map_err(|e| Error::io(IoOp::Delete, &bak_filepath, e))?;
    }

    // wipe the .bak if it exists as well
    let bak_filepath = dest_dir
        .join(source_file.filename_str().expect(E_FILENAME))
        .append_extension(BAK);

    if bak_filepath.exists() {
        std::fs::remove_file(&bak_filepath)
            .map_err(|e| Error::io(IoOp::Delete, &bak_filepath, e))?;
    }

    Ok(())
}

/// Retrieves a list of all `.bak.N` files in the directory.
fn list_bak_n_files(file: &Path, dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let bak_n_file_pattern = file
        .append_extension(BAK_DOT)
        .filename_string().expect(E_FILENAME);

    let dir = dir.read_dir()
        .map_err(|e| Error::io(IoOp::Read, dir, e))?;
    let mut paths: Vec<PathBuf> = dir
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

    paths.sort();

    Ok(paths)
}

/// Performs a copy
fn run_backup(cli: &cli::Cli) -> Result<(), Error> {
    let dir = cli.dir();
    let app_data_dir = os::user_app_data_dir(true, BAK9.into())
        .map_err(|e| Error::Generic(e.to_string()))?;
    let is_app_data_dir = dir == app_data_dir;

    let dest_dir = if is_app_data_dir {
        mirror_dir(&app_data_dir, &cli.file, true)?
    } else {
        dir
    };

    let bak_filepath = match determine_destination(&cli.file, &dest_dir, cli.num)? {
        Some(filepath) => filepath,
        None => return Ok(())
    };

    match os::copy_file(&cli.file, &bak_filepath) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied && !is_app_data_dir => {
            let app_data_dir = os::user_app_data_dir(true, BAK9.into())
                .map_err(|e| Error::Generic(e.to_string()))?;

            let mirror_dir = mirror_dir(&app_data_dir, &cli.file, true)?;

            let home_bak_filepath = match determine_destination(&cli.file, &mirror_dir, cli.num)? {
                Some(filepath) => filepath,
                None => return Ok(())
            };

            os::copy_file(&cli.file, &home_bak_filepath)
                .map_err(|_| Error::copy(&cli.file, &home_bak_filepath, e))?;

            if !cli.quiet {
                eprintln!("{} copied to {}", "notice:".yellow(),
                    sanitize_path_str(home_bak_filepath.to_str().expect(E_STR)).cyan());
            }

            Ok(())
        },
        Err(e) => Err(Error::copy(&cli.file, &bak_filepath, e))
    }
}

fn determine_destination(source_file: &Path, dest_dir: &Path, max: u8) -> Result<Option<PathBuf>, Error> {
    let source_filename = source_file.filename_str().expect(E_FILENAME);
    let last_bak = find_last_bak(source_file, dest_dir);

    if let Some(last_bak_filepath) = &last_bak {
        if !diff_files(last_bak_filepath, source_file)? {
            return Ok(None)
        }
    }

    let bak_filepath = if let Some(last_bak_filepath) = &last_bak {
        if max == 1 {
            wipe(source_file, dest_dir)?;

            dest_dir
                .join(&source_filename)
                .append_extension(BAK)
        } else if last_bak_filepath.extension().expect("Expected .bak") == BAK {
            shift_bak_files(&source_file, &dest_dir, max)?;

            let bak1_filepath = dest_dir
                .join(&source_filename)
                .append_extension(BAK_1);

            fs::rename(last_bak_filepath, &bak1_filepath)
                .map_err(|e| Error::io(IoOp::Rename, &bak1_filepath, e))?;

            dest_dir
                .join(&source_filename)
                .append_extension(BAK_0)
        } else {
            shift_bak_files(source_file, dest_dir, max)?;

            dest_dir 
                .join(&source_filename)
                .append_extension(BAK_0)
        }
    } else { 
        dest_dir
            .join(&source_filename)
            .append_extension(BAK)
    };

    Ok(Some(bak_filepath))
}

pub fn sanitize_path_str(path: &str) -> &str {
    sanitize_windows_path_str(path)
}

pub fn sanitize_windows_path_str(path: &str) -> &str {
    path.trim_start_matches("\\\\?\\")
}

fn determine_mirror_dir(base_dir: &Path, src_file: &Path) -> Result<PathBuf, Error> {
    let src_dir = src_file.parent().expect("Expected parent directory");
    let mut mirror_dir = base_dir.to_path_buf();

    for component in src_dir.components() {
        let dirname = component.as_os_str().to_str().expect(E_STR)
            .trim_start_matches("\\\\?\\"); // remove any windows extended path prefix

        match dirname {
            "." | "/" | "\\" => continue,
            ".." => unreachable!("Expected absolute path"),
            _ => {}
        }

        // windows drives (C:, D:, etc)
        if dirname.chars().count() == 2 && dirname.chars().nth(1).unwrap() == ':' {
            mirror_dir.push(dirname.chars().nth(0).unwrap().to_string());
        } else {
            mirror_dir.push(dirname);
        }
    }

    Ok(mirror_dir)
}

pub fn mirror_dir(base_dir: &Path, src_file: &Path, mkdir: bool) -> Result<PathBuf, Error> {
    let mirror_dir = determine_mirror_dir(base_dir, src_file)?;

    if !mirror_dir.is_dir() && mkdir {
        fs::create_dir_all(&mirror_dir)
            .map_err(|e| Error::io(IoOp::Create, &mirror_dir, e))?
    }

    mirror_dir.canonicalize()
        .map_err(|e| Error::io(IoOp::Read, &mirror_dir, e))
}

fn diff_files(a: &Path, b: &Path) -> Result<bool, Error> {
    // check to see if a backup is necessary, using a file diff
    let mut file = std::fs::File::open(a)
        .map_err(|e| Error::io(IoOp::Read, a, e))?;
    let mut last_bak = std::fs::File::open(b)
        .map_err(|e| Error::io(IoOp::Read, b, e))?;

    Ok(!file_diff::diff_files(&mut file, &mut last_bak))
}

/// Increments the filename extension of all `.bak.N` files in the directory.
fn shift_bak_files(file: &Path, dir: &Path, num: u8) -> Result<(), Error> {
    let mut bak_filepaths = list_bak_n_files(file, dir)?;

    // prune all excess backups
    if bak_filepaths.len() >= num as usize {
        let prune_amount = bak_filepaths.len() - num as usize + 1;
        for _i in 0..prune_amount {
            let bak_filepath = bak_filepaths.pop().expect("Expected array value");
            std::fs::remove_file(&bak_filepath)
                .map_err(|e| Error::io(IoOp::Delete, &bak_filepath, e))?;
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
            .map_err(|e| Error::io(IoOp::Write, &bak_next_filepath, e))?;
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn test_determine_mirror_dir() {
        let base_dir = "/home/dev/.local/share/bak9";
        let src_file = "/home/dev/tmp/source.txt";
        let mirror_dir = determine_mirror_dir(Path::new(base_dir), Path::new(src_file)).unwrap();
        assert_eq!(Path::new("/home/dev/.local/share/bak9/home/dev/tmp"), mirror_dir);
    }
    
    #[cfg(target_os = "windows")]
    #[test]
    fn test_determine_mirror_dir_windows() {
        // test a windows path with path extensions
        let base_dir = "\\\\?\\C:\\Users\\dev\\AppData\\Local\\bak9";
        let src_file = "\\\\?\\C:\\Users\\dev\\tmp\\source.txt";
        let mirror_dir = determine_mirror_dir(Path::new(base_dir), Path::new(src_file)).unwrap();
        assert_eq!("\\\\?\\C:\\Users\\dev\\AppData\\Local\\bak9\\C\\Users\\dev\\tmp", mirror_dir.to_str().unwrap());
    }
}