use std::path::PathBuf;
use file_diff;

pub use function_name::named;

const TESTS: &str = "tests";

pub fn open_tmpdir(subdir: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir);

    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .expect("Failed to remove existing directory");
    }

    std::fs::create_dir_all(&dir)
        .expect("Failed to create directory");

    dir
}

pub fn open_tmpdir_topic(topic: &str, subdir: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir)
        .join(topic);

    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .expect("Failed to remove existing directory");
    }

    std::fs::create_dir_all(&dir)
        .expect("Failed to create directory");

    dir
}

pub fn close_tmpdir_topic(topic: &str, subdir: &str) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir)
        .join(topic);

    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .expect("Failed to remove existing directory");
    }
}

pub fn close_tmpdir(subdir: &str) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir);

    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .expect("Failed to remove existing directory");
    }
}

pub fn tmpfile_exists(filename: &str, subdir: &str) -> bool {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir)
        .join(filename)
        .exists()
}

pub fn tmpfile_topic_exists(filename: &str, topic: &str, subdir: &str) -> bool {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir)
        .join(topic)
        .join(filename)
        .exists()
}

pub fn tmpfile_append(content: &str, filename: &str, subdir: &str) -> PathBuf {
    use std::io::prelude::*;

    let filepath = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir)
        .join(filename);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filepath)
        .unwrap();

    writeln!(file, "{}", content).unwrap();

    filepath
}

/// Returns true if they are different
pub fn tmpfile_diff(filename_a: &str, filename_b: &str, subdir: &str) -> bool {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(TESTS)
        .join(subdir);

    let mut file_a = std::fs::File::open(dir.join(filename_a)).unwrap();
    let mut file_b = std::fs::File::open(dir.join(filename_b)).unwrap();

    !file_diff::diff_files(&mut file_a, &mut file_b)
}
