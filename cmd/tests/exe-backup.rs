mod testlib;

#[cfg(test)]
mod tests {
    use std::process;
    use asmov_testing::{self as testing, prelude::*};
    use bak9::paths;
    use super::testlib::{self, TestlibModuleBuilder};

    const BIN_EXE: &str = env!("CARGO_BIN_EXE_bak9");

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .testlib_module_defaults()
            .using_temp_dir()
            .build()
    });

    fn exe_bak9<S: AsRef<std::ffi::OsStr>>(test: &testing::Test, assert_success: Option<bool>, args: &[S]) -> (String, String) {
        let mock_root = test.imported_fixture_dir(&testlib::NAMEPATH).join(testlib::MOCK_FS_DIRNAME); 
        let output = process::Command::new(BIN_EXE)
            .args(args)
            .env(paths::ENV_BAK9_HOME, mock_root.join(testlib::FIXTURE_USER_HOME_PATHSTR))
            .env(testlib::ENV_BAK9_TEST_MOCK_ROOT_DIR, &mock_root)
            .env(testlib::ENV_BAK9_TEST_TMP_DIR, test.temp_dir())
            .output()
            .unwrap();

        if let Some(success) = assert_success {
            assert_eq!(success, output.status.success());
        }

        (String::from_utf8(output.stdout).unwrap(), String::from_utf8(output.stderr).unwrap())
    }

    #[named]
    #[test]
    fn test_help() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .build();

        let (stdout, stderr) = exe_bak9(&test, Some(true), &["backup", "--help"]);
        assert!(stdout.contains("Usage: bak9 backup "));
        assert_eq!("", stderr);
    }

    fn setup_backup_dir(test: &mut testing::Test) {
        paths::setup_backup_storage_dir(&test.temp_dir().join("strg").join("backup")).unwrap();
    }

    #[named]
    #[test]
    fn test_scheduled() {
        let test = TESTING.test(function_name!())
            .using_temp_dir()
            .setup(setup_backup_dir)
            .build();

        let (stdout, stderr) = exe_bak9(&test, Some(false), &["backup", "scheduled"]);
        assert_eq!("", stdout);
        assert_eq!("", stderr);
    }
}