
#[cfg(test)]
mod tests {
    use bak9;
    use std::path::PathBuf;
    use function_name::named;
    use file_diff;

    const TESTS: &str = "tests";

    fn open_tmpdir(subdir: &str) -> PathBuf {
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

    fn open_tmpdir_topic(topic: &str, subdir: &str) -> PathBuf {
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

    fn close_tmpdir_topic(topic: &str, subdir: &str) {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(TESTS)
            .join(subdir)
            .join(topic);

        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .expect("Failed to remove existing directory");
        }
    }

    fn close_tmpdir(subdir: &str) {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(TESTS)
            .join(subdir);

        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .expect("Failed to remove existing directory");
        }
    }

    fn tmpfile_exists(filename: &str, subdir: &str) -> bool {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(TESTS)
            .join(subdir)
            .join(filename)
            .exists()
    }

    fn tmpfile_topic_exists(filename: &str, topic: &str, subdir: &str) -> bool {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(TESTS)
            .join(subdir)
            .join(topic)
            .join(filename)
            .exists()
    }

    fn tmpfile_append(content: &str, filename: &str, subdir: &str) {
        use std::io::prelude::*;

        let filepath = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(TESTS)
            .join(subdir)
            .join(filename);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)
            .unwrap();

        writeln!(file, "{}", content).unwrap();
    }

    /// Returns true if they are different
    fn tmpfile_diff(filename_a: &str, filename_b: &str, subdir: &str) -> bool {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(TESTS)
            .join(subdir);

        let mut file_a = std::fs::File::open(dir.join(filename_a)).unwrap();
        let mut file_b = std::fs::File::open(dir.join(filename_b)).unwrap();

        !file_diff::diff_files(&mut file_a, &mut file_b)
    }

    #[test]
    #[named]
    fn test_no_extension() {
        let tmpdir = open_tmpdir(function_name!());

        std::fs::write(tmpdir.join("no_extension"), "LINE 1").unwrap();
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("no_extension"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("no_extension.bak", function_name!()),
            "no_extension should be created");
    }
 
    #[test]
    #[named]
    fn test_chain() {
        let tmpdir = open_tmpdir(function_name!());

        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("noexist.txt"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_err());

        // source_1

        std::fs::write(tmpdir.join("source_1.txt"), "LINE 1").unwrap();
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak", function_name!()),
            "source_1.txt.bak should be created");
        
        tmpfile_append("LINE 2", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));
        assert_eq!(false, tmpfile_diff("source_1.txt", "source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_diff("source_1.txt", "source_1.txt.bak.1", function_name!()));

        tmpfile_append("LINE 3", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.2", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));
        assert_eq!(false, tmpfile_diff("source_1.txt", "source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_diff("source_1.txt", "source_1.txt.bak.1", function_name!()));
        assert_eq!(true, tmpfile_diff("source_1.txt", "source_1.txt.bak.2", function_name!()));
        assert_eq!(true, tmpfile_diff("source_1.txt.bak.1", "source_1.txt.bak.2", function_name!()));

        tmpfile_append("LINE 4", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.2", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.3", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));

        tmpfile_append("LINE 5", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 2,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.2", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.3", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));

        tmpfile_append("LINE 6", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 1,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak", function_name!()));
        assert_eq!(false, tmpfile_diff("source_1.txt", "source_1.txt.bak", function_name!()));

        tmpfile_append("LINE 7", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));
 
        tmpfile_append("LINE 8", "source_1.txt", function_name!());
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            delete: true,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));

        close_tmpdir(function_name!());
    }

    #[test]
    #[named]
    fn test_other_dir() {
        let tmpdir = open_tmpdir(function_name!());
        
        let topic_tmpdir = open_tmpdir_topic("source_2_dir", function_name!());
        std::fs::write(tmpdir.join("source_2.txt"), "LINE 1").unwrap();
        let result = bak9::run_with(bak9::cli::Cli {
            file: tmpdir.join("source_2.txt"),
            dir: Some(topic_tmpdir),
            delete: false,
            num: 3,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_topic_exists("source_2.txt.bak", "source_2_dir", function_name!()));
        close_tmpdir_topic("source_2_dir", function_name!());

        close_tmpdir(function_name!());
    }
}