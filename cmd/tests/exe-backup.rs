mod test_common;

#[cfg(test)]
mod tests {
    use std::process;
    use asmov_testing::{self as testing, prelude::*};
    use bak9::paths;
    use super::test_common;

    const BIN_EXE: &str = env!("CARGO_BIN_EXE_bak9");

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .import_fixture_dir(&*test_common::NAMEPATH)
            .setup(test_common::setup_env)
            .using_temp_dir()
            .build()
    });

    fn exe_bak9<S: AsRef<std::ffi::OsStr>>(test: &testing::Test, assert_success: Option<bool>, args: &[S]) -> (String, String) {
        let output = process::Command::new(BIN_EXE)
            .args(args)
            .env(paths::BAK9_HOME,
                test.imported_fixture_dir(&*test_common::NAMEPATH)
                    .join(test_common::FIXTURE_USER_HOME))
            .env(test_common::BAK9_TMP_DIR, test.temp_dir())
            .env(test_common::BAK9_TESTS_DIR, test.imported_fixture_dir(&*test_common::NAMEPATH))
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