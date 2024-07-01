mod testlib;

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, process, sync::OnceLock};
    use asmov_testing::{self as testing, prelude::*};
    use bak9::paths::{self, Bak9Path};
    use super::testlib::{self, TestlibModuleBuilder};

    const BIN_EXE: &str = env!("CARGO_BIN_EXE_bak9");

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .testlib_module_defaults()
            .using_temp_dir()
            .build()
    });

    fn exe_bak9<S>(
        test: &testing::Test,
        source_num: u8,
        assert_success: Option<bool>,
        args: &[S]
    ) -> (String, String)
    where
        S: AsRef<std::ffi::OsStr>
    {
        let mock_root = test.imported_fixture_dir(&testlib::testlib_namepath())
            .join(testlib::MOCK_FS_DIRNAME)
            .join(format!("{}{source_num}", testlib::SOURCE_PREFIX)); 
        let output = process::Command::new(BIN_EXE)
            .args(args)
            .env(paths::consts::ENV_BAK9_HOME, mock_root.join(testlib::HOME_TESTUSR))
            .env(testlib::ENV_BAK9_TEST_SOURCE_ROOT, &mock_root)
            .env(testlib::ENV_BAK9_TEST_TMP_DIR, test.temp_dir())
            .output()
            .unwrap();

        let stdout = strip_ansi_escapes::strip_str(String::from_utf8(output.stdout).unwrap());
        let stderr = strip_ansi_escapes::strip_str(String::from_utf8(output.stderr).unwrap());

        if let Some(success) = assert_success {
            assert_eq!(success, output.status.success(), "Command failed. stdout: {stdout} :: stderr: {stderr}");
        }

        (stdout, stderr)
    }

    /// Returns the (name, dest_dir) of each successful backup.
    fn parse_stdout_backup_results<'stdout>(stdout: &'stdout str) -> Vec<(&'stdout str, PathBuf)> {
        const RE: &str = r"(?m)bak9] Completed (?:full|incremental) backup of ([^ ]+) to (.+)$";
        static REGEX: OnceLock<regex::Regex> = OnceLock::new();
        let regex = REGEX.get_or_init(|| regex::Regex::new(RE).unwrap());

        let mut results = Vec::new();
        for (_, [name, dest_dir]) in regex.captures_iter(stdout).map(|c| c.extract()) {
            results.push((name, PathBuf::from(dest_dir).canonicalize().unwrap()));
        }

        results
    }

    #[named]
    #[test]
    fn test_help() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .build();

        let (stdout, stderr) = exe_bak9(&test, 1, Some(true), &["backup", "--help"]);
        assert!(stdout.contains("Usage: bak9 backup "));
        assert_eq!("", stderr);
    }

    fn setup_backup_dir(test: &mut testing::Test) {
        Bak9Path::StorageDir(test.temp_dir().join("strg/backup")).setup().unwrap();
    }

    #[named]
    #[test]
    fn test_scheduled() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .setup(setup_backup_dir)
            .build();

        let (stdout, stderr) = exe_bak9(&test, 1, Some(true), &["backup", "scheduled"]);
        assert_eq!("", stdout);
        assert_eq!("", stderr);
    }

    #[named]
    #[test]
    fn test_manual_full() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .setup(setup_backup_dir)
            .build();

        let source_1_dir = testlib::source_dir(1, &test);
 
        let (stdout, stderr) = exe_bak9(&test, 1, Some(true), &["backup", "full", "home-testusr"]);
        assert_eq!("", stderr);
        let results = parse_stdout_backup_results(&stdout);
        assert_eq!(1, results.len());
        assert_eq!("home-testusr", results[0].0);
        assert!(!dir_diff::is_different(&source_1_dir, &results[0].1.join(testlib::TESTUSR)).unwrap());
    }

    #[named]
    #[test]
    fn test_manual_incremental() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .setup(setup_backup_dir)
            .build();

        let source_1_dir = testlib::source_dir(1, &test);
        let source_2_dir = testlib::source_dir(2, &test);
 
        let (stdout, stderr) = exe_bak9(&test, 1, Some(true), &["backup", "full", "home-testusr"]);
        assert_eq!("", stderr);
        let results = parse_stdout_backup_results(&stdout);
        assert_eq!(1, results.len());
        assert_eq!("home-testusr", results[0].0);
        assert!(!dir_diff::is_different(&source_1_dir, &results[0].1.join(testlib::TESTUSR)).unwrap());

        let (stdout, stderr) = exe_bak9(&test, 2, Some(true), &["backup", "incremental", "home-testusr"]);
        assert_eq!("", stderr);
        let results = parse_stdout_backup_results(&stdout);
        assert_eq!(1, results.len());
        assert_eq!("home-testusr", results[0].0);
        assert!(!dir_diff::is_different(&source_2_dir, &results[0].1.join(testlib::TESTUSR)).unwrap());
    }
}