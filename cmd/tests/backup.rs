use std::{fs, path::{Path, PathBuf}};
use chrono;
use hostname;
use cron;


#[cfg(test)]
mod tests {
    use std::os::unix::fs::MetadataExt;
    use chrono::NaiveTime;
    use asmov_testing as testing;
    use bak9::{config, paths, backup, schedule, cmd::{rsync, xz}};
    use function_name::named;
    use super::*;

    pub const TEST_HOME_DIR: &'static str = "tests/fixtures/fs/home/testusr";
    pub const BAK9_TMP_DIR: &'static str = "BAK9_TMP_DIR";
    pub const BAK9_TESTS_DIR: &'static str = "BAK9_TESTS_DIR";
 
    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .base_temp_dir(env!("CARGO_TARGET_TMPDIR"))
            .using_temp_dir()
            .setup(|_| {
                std::env::set_var(paths::BAK9_HOME,
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join(TEST_HOME_DIR)
                );

                std::env::set_var(BAK9_TMP_DIR, PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
                std::env::set_var(BAK9_TESTS_DIR, PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"));
            })
            .build()
    });

    fn read_config(tmp_dir: &Path) -> config::BackupConfig {
        let config_path = paths::home_dir().unwrap()
            .join(paths::HOME_CONFIG_DIR)
            .join(paths::BAK9_CONFIG_FILENAME);

        let mut config = config::BackupConfig::read(&config_path).unwrap();
        
        config.backup_storage_dir = shellexpand::env_with_context(&config.backup_storage_dir, |var| {
            match var.as_ref() {
                "BAK9_TMP_DIR" => Ok(Some(tmp_dir.to_str().unwrap().to_string())),
                _ => Err(std::env::VarError::NotPresent)
            }
        }).unwrap().to_string();

        config
    }


    fn cmd_diff(source_dir: &Path, dest_dir: &Path) -> std::process::Command {
        let mut cmd = std::process::Command::new("diff");
        let dest_dir = dest_dir.join(source_dir.file_name().unwrap());
        cmd.args(&[
            "-q",
            source_dir.to_str().unwrap(),
            dest_dir.to_str().unwrap(),
        ]);
        cmd
    }

    // verify that all files were hardlinked from the previous backup
    fn verify_hardlinked(dir: &Path) {
        for entry in fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            if metadata.is_file() {
                assert!(metadata.nlink() > 1);
            } else {
                verify_hardlinked(&entry.path());
            }
        }
    }

    fn _setup_tmp_dirs(config: &config::BackupConfig) {
        paths::setup_backup_storage_dir(config).unwrap();
    }

    #[named]
    #[test]
    fn test_read_config() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .build();

        let tmp_dir = test.temp_dir();
        let config = read_config(tmp_dir);
        //dbg!(&config);
        assert_eq!(config.backup_storage_dir, format!("{}/strg/backup", tmp_dir.to_str().unwrap()));
        assert_eq!(config.backup("home-testusr").unwrap().archives.len(), 2);
        assert_eq!(config.backup("home-testusr").unwrap().archive("quarterly").unwrap().max_archives, 4);
    }

    #[named]
    #[test]
    fn test_setup_backup_dirs() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .build();

        let config = read_config(test.temp_dir());
        paths::setup_backup_storage_dir(&config).unwrap();
        assert!(config.backup_storage_dir_path().exists());
        assert!(config.backup_storage_dir_path().join(paths::BACKUP_ARCHIVE_DIRNAME).exists());
        assert!(config.backup_storage_dir_path().join(paths::BACKUP_FULL_DIRNAME).exists());
        assert!(config.backup_storage_dir_path().join(paths::BACKUP_INCREMENTAL_DIRNAME).exists());
        assert!(config.backup_storage_dir_path().join(paths::BACKUP_LOGS_DIRNAME).exists());
    }

    fn do_backup_full(name: &str, config: &config::BackupConfig, host: &str) -> PathBuf {
        let backup_cfg = config.backup(name).unwrap();
        let run_name = backup::backup_run_name(&schedule::datetimestamp_today(), &backup_cfg.name, &host);
        let source_dir = backup_cfg.source_dir_path();
        let dest_dir = config.backup_storage_dir_path()
            .join(paths::BACKUP_FULL_DIRNAME)
            .join(&run_name);

        // perform a full backup
        let mut rsync_cmd = rsync::cmd_rsync_full(&source_dir, &dest_dir);
        let started = std::time::Instant::now();
        let output = rsync_cmd.output().unwrap();
        dbg!(&output);
        assert!(output.status.success());

        // verify that the full backup is correct
        do_diff(&source_dir, &dest_dir);

        if started.elapsed().as_millis() < 1 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        dest_dir
    }

    // perform an incremental backup off of the previous one
    fn do_backup_incremental(name: &str, config: &config::BackupConfig, host: &str, prev_full_dir: &Path) -> PathBuf {
        let backup_cfg = config.backup(name).unwrap();
        let source_dir = backup_cfg.source_dir_path();
        let run_name = backup::backup_run_name(&schedule::datetimestamp_today(), &backup_cfg.name, &host);
        let dest_dir = config.backup_storage_dir_path()
            .join(paths::BACKUP_INCREMENTAL_DIRNAME)
            .join(&run_name);
        let mut rsync_cmd = rsync::cmd_rsync_incremental(prev_full_dir, &source_dir, &dest_dir);
        let started = std::time::Instant::now();
        let output = rsync_cmd.output().unwrap();
        dbg!(&output);
        assert!(output.status.success());

        // verify that the incremental backup is correct
        do_diff(&source_dir, &dest_dir);

        // make sure that all files were hardlinked on the last incremental run
        verify_hardlinked(&dest_dir);
        
        if started.elapsed().as_millis() < 1 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        dest_dir
    }

    fn do_zip(backup_name: &str, config: &config::BackupConfig, host: &str, full_dir: &Path) -> PathBuf {
        let backup_cfg = config.backup(backup_name).unwrap();
        let _source_dir = backup_cfg.source_dir_path();
        let run_name = backup::backup_run_name(&schedule::datetimestamp_today(), &backup_cfg.name, &host);
         let zip_file = config.backup_storage_dir_path()
            .join(paths::BACKUP_ARCHIVE_DIRNAME)
            .join(&run_name)
            .with_extension("tar.xz");
        let mut tar_xz_cmd = xz::cmd_tar_xz(&full_dir, &zip_file);
        let output = tar_xz_cmd.output().unwrap();
        dbg!(&output);
        assert!(output.status.success());

        //todo: unzip and diff the archive against the full backup

        zip_file
    }

    fn do_diff(source_dir: &Path, dest_dir: &Path) {
        let mut diff_cmd = cmd_diff(&source_dir, &dest_dir);
        let output = diff_cmd.output().unwrap();
        assert!(output.status.success());
    }

    #[named]
    #[test]
    fn test_backup_and_archive_mechanics() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .build();

        let config = read_config(test.temp_dir());
        paths::setup_backup_storage_dir(&config).unwrap();
        let host = hostname::get().unwrap().into_string().unwrap();

        for backup_cfg in &config.backups {
            let backup_name = &backup_cfg.name;
            let full_dir = do_backup_full(&backup_name, &config, &host);

            // zip the full backup
            do_zip(&backup_name, &config, &host, &full_dir);

            // ensure that the timestamp is always different

            // perform an incremental backup off of the previous one
            do_backup_incremental(&backup_name, &config, &host, &full_dir);
        }
    }

    #[named]
    #[test]
    fn test_scheduling_logic() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .build();

        let config = read_config(test.temp_dir());
        let host = hostname::get().unwrap().into_string().unwrap();
        paths::setup_backup_storage_dir(&config).unwrap();
        let tomorrow = chrono::Local::now() + chrono::Duration::days(1);
        let expected_next = tomorrow
            .with_time(NaiveTime::from_hms_opt(2, 30, 0).unwrap()).unwrap();

        for backup_cfg in &config.backups {
            do_backup_full(&backup_cfg.name, &config, &host);

            let last_full_backup = backup::find_last_full_backup(&backup_cfg.name, &host, &config).unwrap();
            let last_filename = last_full_backup.file_name().unwrap().to_str().unwrap();
            let (last_backup_time, _, _) = backup::parse_backup_name(&last_filename, &config);

            let schedule_cfg = config.schedule(&backup_cfg.incremental_schedule).unwrap();
            let schedule = cron::Schedule::from(schedule_cfg);
            let next = schedule.after(&last_backup_time).next().unwrap();
            assert_eq!(next, expected_next);
        }
    }
}