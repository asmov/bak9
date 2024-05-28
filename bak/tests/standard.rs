mod common;

#[cfg(test)]
mod tests {
    use super::common::*;
    use bak9_bak as bak;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    #[named]
    fn test_no_extension() {
        let tmpdir = open_tmpdir(function_name!());

        std::fs::write(tmpdir.join("no_extension"), "LINE 1").unwrap();
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("no_extension"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("no_extension.bak", function_name!()),
            "no_extension should be created");
    }
 
    #[test]
    #[named]
    fn test_chain() {
        let tmpdir = open_tmpdir(function_name!());

        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("noexist.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_err());

        //STEP: Backup source_1.txt
        //RESULT: source_1.txt.bak should be created

        std::fs::write(tmpdir.join("source_1.txt"), "LINE 1").unwrap();
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak", function_name!()),
            "source_1.txt.bak should be created");
        
        //STEP: Append to source_1.txt. Backup source_1.txt again 
        //RESULT: source_1.txt.bak should be renamed to source_1.txt.bak.1. source_1.txt.bak.0 should be created

        tmpfile_append("LINE 2", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));
        assert_eq!(false, tmpfile_diff("source_1.txt", "source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_diff("source_1.txt", "source_1.txt.bak.1", function_name!()));

        //STEP: Append to source_1.txt. Backup source_1.txt again
        //RESULT: source_1.txt.bak.0,1,2 should now exist

        tmpfile_append("LINE 3", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
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

        //STEP: Append to source_1.txt. Backup source_1.txt again
        //RESULT: Baks 0,1,2 should exist, the previous .bak.2 should have been pruned out.

        tmpfile_append("LINE 4", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.2", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.3", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));

        tmpfile_append("LINE 5", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 2,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.2", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.3", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));

        tmpfile_append("LINE 6", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 1,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak", function_name!()));
        assert_eq!(false, tmpfile_diff("source_1.txt", "source_1.txt.bak", function_name!()));

        tmpfile_append("LINE 7", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.0", function_name!()));
        assert_eq!(true, tmpfile_exists("source_1.txt.bak.1", function_name!()));
        assert_eq!(false, tmpfile_exists("source_1.txt.bak", function_name!()));

        //STEP: Wipe
        //RESULT: All baks should be removed
 
        tmpfile_append("LINE 8", "source_1.txt", function_name!());
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_1.txt"),
            dir: None,
            num: 3,
            force: true,
            quiet: true,
            subcommand: Some(bak::cli::Command::Wipe),
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
        let result = bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source_2.txt"),
            dir: Some(topic_tmpdir),
            num: 3,
            force: true,
            quiet: true,
            subcommand: None,
        });
        assert_eq!(true, result.is_ok());
        assert_eq!(true, tmpfile_topic_exists("source_2.txt.bak", "source_2_dir", function_name!()));
        close_tmpdir_topic("source_2_dir", function_name!());

        close_tmpdir(function_name!());
    }

    #[named]
    #[test]
    fn test_app_data_dir_mirror() {
        let tmpdir = open_tmpdir(function_name!());
        let source_filepath = tmpfile_append("LINE 1", "source.txt", function_name!());
        bak::run_with(
            bak::cli::Cli::parse_from(["-f", "-q", "-n", "3", source_filepath.to_str().unwrap(), "-"])
        ).unwrap();

        let app_data_dir = bak::os::user_app_data_dir(true, bak::BAK9.into())
            .expect("Failed to get user app data directory");
        let mirror_dir = bak::mirror_dir(&app_data_dir, &tmpdir.join("source.txt"), false).unwrap();

        assert_eq!(true, mirror_dir.is_dir());
        assert_eq!(true, mirror_dir.join("source.txt.bak").is_file());

        bak::run_with(bak::cli::Cli {
            file: tmpdir.join("source.txt"),
            dir: Some(PathBuf::from("-")),
            num: 3,
            force: true,
            quiet: true,
            subcommand: Some(bak::cli::Command::Wipe),
        }).unwrap();

        assert_eq!(false, mirror_dir.join("source.txt.bak").exists());
        assert_eq!(false, mirror_dir.exists());

        close_tmpdir(function_name!());
    }
}