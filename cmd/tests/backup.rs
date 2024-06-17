mod testlib;

#[cfg(test)]
mod tests {
    use std::{process, vec};
    use asmov_testing::{self as testing, prelude::*};
    use bak9::{config::BackupConfigSchedule, paths};
    use super::testlib::{self, TestlibModuleBuilder};

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .testlib_module_defaults()
            .using_temp_dir()
            .build()
    });

    fn setup_backup_dir(test: &mut testing::Test) {
        paths::setup_backup_storage_dir(&test.temp_dir().join("strg").join("backup")).unwrap();
    }

    fn make_cli(_test: &testing::Test) -> bak9::cli::Cli {
        bak9::cli::Cli {
            config_file: None,
            force: false,
            subcommand: bak9::cli::Command::Backup(bak9::cli::BackupCommand::Scheduled),
        }
    }

    fn make_config(test: &testing::Test) -> bak9::config::BackupConfig {
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
                    source_dir: test.imported_fixture_dir(&testlib::NAMEPATH)
                        .join(testlib::MOCK_FS_DIRNAME)
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

    fn bak9_backup(cli: &bak9::cli::Cli, config: &bak9::config::BackupConfig) -> bak9::backup::BackupJobResults {
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
        let config = make_config(&test);

        let results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(1, results.len());
        let result = results.get(0).unwrap();
        assert!(matches!(result, bak9::backup::BackupJobOutput::Full(..)));
        let result = match result {
            bak9::backup::BackupJobOutput::Full(result) => result,
            _ => panic!("unexpected result"),
        };
        assert_eq!(false, dir_diff::is_different(&result.source_dir, &result.dest_dir.join(testlib::TESTUSR)).unwrap());

        // try running it again. it should not create a new backup for "today"
        let results = bak9_backup(&cli, &config).unwrap();
        assert_eq!(0, results.len());
    }
}
