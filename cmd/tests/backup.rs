mod testlib;

#[cfg(test)]
mod tests {
    use std::{fs, vec};
    use std::os::unix::fs::PermissionsExt;
    use asmov_testing::{self as testing, prelude::*};
    use bak9::{config::BackupConfigSchedule, paths, job::*};
    use super::testlib::{self, TestlibModuleBuilder};

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .testlib_module_defaults()
            .using_temp_dir()
            .build()
    });

    fn setup_backup_dir(test: &mut testing::Test) {
        paths::setup_backup_storage_dir(&test.temp_dir().join("strg/backup")).unwrap();
    }

    fn make_cli(_test: &testing::Test) -> bak9::cli::Cli {
        bak9::cli::Cli {
            config_file: None,
            force: false,
            quiet: false,
            subcommand: bak9::cli::Command::Backup(bak9::cli::BackupCommand::Scheduled),
        }
    }

    fn make_config(test: &testing::Test, source_version: u8) -> bak9::config::BackupConfig {
        bak9::config::BackupConfig {
            backup_storage_dir: test.temp_dir().join("strg/backup")
                .to_str().unwrap().to_string(),
            schedules: vec![
                BackupConfigSchedule {
                    name: "daily".to_string(),
                    minute: Some(0),
                    minutes: None,
                    hour: Some(0),
                    hours: None,
                    day_of_week: None,
                    days_of_week: None,
                    day_of_month: None,
                    days_of_month: None,
                    month: None,
                    months: None
                },
                BackupConfigSchedule {
                    name: "monthly".to_string(),
                    minute: Some(0),
                    minutes: None,
                    hour: Some(0),
                    hours: None,
                    day_of_week: None,
                    days_of_week: None,
                    day_of_month: Some(1),
                    days_of_month: None,
                    month: None,
                    months: None
                },

            ],
            backups: vec![
                bak9::config::BackupConfigBackup {
                    name: "home".to_string(),
                    source_dir: test.imported_fixture_dir(&testlib::testlib_namepath())
                        .join(testlib::MOCK_FS_DIRNAME)
                        .join(format!("{}{source_version}", testlib::SOURCE_PREFIX))
                        .join(testlib::HOME_TESTUSR)
                        .to_str().unwrap().to_string(),
                    full_schedule: "monthly".to_string(),
                    incremental_schedule: "daily".to_string(),
                    max_full: 4,
                    archives: vec![], 
                },
            ],
        }
    }

    fn bak9_backup(cli: &bak9::cli::Cli, config: &bak9::config::BackupConfig) -> bak9::backup::JobResults {
        bak9::run::backup::run_backup(&cli, &bak9::cli::BackupCommand::Scheduled, Some(config))
    }

    #[named]
    #[test]
    fn test_scheduled_full() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .setup(setup_backup_dir)
            .build();

        let cli = make_cli(&test);
        let config = make_config(&test, 1);

        let mut results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(2, results.len());
        let archive_output = match results.pop().unwrap() {
            JobOutput::Archive(job) => job, _ => panic!() };
        let backup_output = match results.pop().unwrap() {
            JobOutput::Backup(job) => job, _ => panic!() };
        assert_eq!(false, dir_diff::is_different(
            &backup_output.source_dir,
            &backup_output.dest_dir.join(testlib::TESTUSR)).unwrap());

        // try running it again. it should not create a new backup for "today"
        let results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(0, results.len());
    }

    fn setup_incremental_backup_test(test: &mut testing::Test) {
        setup_backup_dir(test);

        let cli = make_cli(test);
        let config = make_config(test, 1);
        let mut results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(2, results.len());
        results.pop().unwrap();
        let backup_output = match results.pop().unwrap() {
            JobOutput::Backup(job) => job, _ => panic!() };

        let yesterday_run_name = bak9::backup::BackupRunName::new(
            chrono::Local::now().checked_sub_signed(chrono::Duration::days(1)).unwrap(),
            bak9::backup::hostname(),
            bak9::backup::username(),
            &config.backups[0].name,
        );

        let yesterday_backup_run_dir = test.temp_dir().join("strg/backup")
            .join(paths::BACKUP_FULL_DIRNAME)
            .join(yesterday_run_name.to_string());

        fs::rename(&backup_output.dest_dir, yesterday_backup_run_dir).unwrap();
    }

    #[named]
    #[test]
    fn test_scheduled_incremental() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .setup(setup_incremental_backup_test)
            .build();

        let cli = make_cli(&test);
        let config = make_config(&test, 2);

        let mut results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(1, results.len());
        let backup_output = match results.pop().unwrap() {
            JobOutput::Backup(job) => job, _ => panic!() };
 
        let dest_home_dir = backup_output.dest_dir.join(testlib::TESTUSR);
        assert!(!dir_diff::is_different(&backup_output.source_dir, &dest_home_dir).unwrap());
        assert!(dir_diff::is_different(&backup_output.dest_dir, &dest_home_dir).unwrap());
        assert!(dest_home_dir.join("source-2.txt").exists(),
            "New file: source-2.txt");
        assert_eq!("source-2 delta", fs::read_to_string(dest_home_dir.join("delta.txt")).unwrap(),
            "Modified file: delta.txt");
        assert_eq!(0o660, fs::metadata(dest_home_dir.join("alpha").join("alpha.txt")).unwrap().permissions().mode() & 0o777,
            "Modified permissions: alpha/alpha.txt");

        // try running it again. it should not create a new backup for "today"
        let results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(0, results.len());
    }
}
