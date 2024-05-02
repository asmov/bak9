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

mod cli;

use std::{fs, path::{Path, PathBuf}};
use clap::Parser;
use file_diff;

const BAK: &str = "bak";
const BAK_DOT: &str = "bak.";
const BAK_0: &str = "bak.0";
const BAK_1: &str = "bak.1";

trait PathExt {
    fn append_extension(self, ext: &str) -> PathBuf;
    fn filename_string(self) -> String;
}

impl PathExt for PathBuf {
    fn append_extension(self, ext: &str) -> PathBuf {
        self.with_extension(format!("{}.{}", self.extension().unwrap().to_str().unwrap(), ext))
    }

    fn filename_string<'s>(self) -> String {
        self.file_name().unwrap().to_str().unwrap().to_owned()
    }
}

impl PathExt for &Path {
    fn append_extension(self, ext: &str) -> PathBuf {
        self.with_extension(format!("{}.{}", self.extension().unwrap().to_str().unwrap(), ext))
    }

    fn filename_string(self) -> String {
        self.file_name().unwrap().to_str().unwrap().to_owned()
    }
}

pub fn run() {
    let cli = cli::Cli::parse();

    if cli.delete {
        run_delete(&cli)
    } else {
        run_backup(&cli)
    }
}

fn run_delete(cli: &cli::Cli) {
    let bak_filename = cli.file.as_path().append_extension(BAK).filename_string();
    let bak_n_filename = cli.file.as_path().append_extension(BAK_DOT).filename_string();

    let bak_filepaths: Vec<PathBuf> = cli.dir()
        .read_dir().unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .filter(|filepath| {
            let filename = filepath.file_name().unwrap().to_str().unwrap();
            if filename == &bak_filename {
                true
            } else if filename.starts_with(&bak_n_filename) {
                matches!(filename.trim_start_matches(&bak_n_filename).parse::<u8>(), Ok(_))
            } else {
                false
            }
        })
        .collect();

    for bak_filepath in bak_filepaths {
        std::fs::remove_file(bak_filepath).unwrap();
    }
}

fn run_backup(cli: &cli::Cli) {
    let last_bak = find_last_bak(&cli.file, &cli.dir());

    // check to see if a backup is necessary, using a file diff
    if let Some(last_bak_filepath) = &last_bak {
        let mut file = std::fs::File::open(&cli.file).unwrap();
        let mut last_bak = std::fs::File::open(&last_bak_filepath).unwrap();
        if file_diff::diff_files(&mut file, &mut last_bak) {
            return; // skip
        }
    }

    let bak_filepath = if let Some(last_bak_filepath) = &last_bak {
        if cli.num == 1 {
            run_delete(cli);

            cli.dir()
                .join(cli.file.file_name().unwrap())
                .append_extension(BAK)
        } else if last_bak_filepath.extension().unwrap() == BAK {
            shift_bak_files(&cli.file, &cli.dir(), cli.num);

            let bak1_filepath = cli.dir()
                .join(cli.file.file_name().unwrap())
                .append_extension(BAK_1);
            fs::rename(last_bak_filepath, bak1_filepath).unwrap();

            cli.dir()
                .join(cli.file.file_name().unwrap())
                .append_extension(BAK_0)
        } else {
            shift_bak_files(&cli.file, &cli.dir(), cli.num);

            cli.dir()
                .join(cli.file.file_name().unwrap())
                .append_extension(BAK_0)
        }
    } else { 
        cli.dir()
            .join(cli.file.file_name().unwrap())
            .append_extension(BAK)
    };

    fs::copy(&cli.file, bak_filepath).unwrap();
}

fn shift_bak_files(file: &Path, dir: &Path, num: u8) {
    let bak_n_file_pattern = file
        .append_extension(BAK_DOT);
    let bak_n_file_pattern = bak_n_file_pattern
        .file_name().unwrap().to_str().unwrap();

    let mut bak_filepaths: Vec<PathBuf> = dir.read_dir().unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .filter(|filepath| {
            let filename = filepath.file_name().unwrap().to_str().unwrap();
            if filename.starts_with(bak_n_file_pattern) {
                matches!(filename.trim_start_matches(bak_n_file_pattern).parse::<u8>(), Ok(_))
            } else {
                false
            }
        })
        .collect();

    // 0, 1, ..
    bak_filepaths.sort_by(|a, b| a.cmp(b));

    // prune all excess backups
    if bak_filepaths.len() >= num as usize {
        let prune_amount = bak_filepaths.len() - num as usize + 1;
        for _i in 0..prune_amount {
            let bak_filepath = bak_filepaths.pop().unwrap();
            std::fs::remove_file(bak_filepath).unwrap();
        }
    }

    // shift each up by 1
    for bak_filepath in bak_filepaths.into_iter().rev() {
        let n = bak_filepath.file_name().unwrap().to_str().unwrap()
            .trim_start_matches(bak_n_file_pattern)
            .parse::<u8>().unwrap();
        let bak_next_filepath = dir.join(file.file_name().unwrap())
            .append_extension(BAK)
            .append_extension((n + 1).to_string().as_str());
        fs::rename(bak_filepath, bak_next_filepath).unwrap();
    }
}

fn find_last_bak(file: &Path, dir: &Path) -> Option<PathBuf> {
    let bak_file = dir.join(file.file_name().unwrap())
        .append_extension(BAK);
    if bak_file.exists() {
        Some(bak_file)
    } else {
        let bak0_file = dir.join(file.file_name().unwrap()).append_extension(BAK_0);
        if bak0_file.exists() {
            Some(bak0_file)
        } else {
            None
        }
    }
}
