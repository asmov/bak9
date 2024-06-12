mod test_common;

#[cfg(test)]
mod tests {
    use std::{process, path::PathBuf};
    use asmov_testing::{self as testing, named};
    use bak9::paths;
    use super::test_common::{self, COMMON_TESTING};

    const BIN_EXE: &str = env!("CARGO_BIN_EXE_bak9");

    static TESTING: testing::StaticGroup = testing::group(|| {
        COMMON_TESTING.group("exe-backup")
            .inherit_fixture_dir()
            .using_temp_dir()
            .build()
    });

    fn cmd<S: AsRef<std::ffi::OsStr>>(test: &testing::Test, success: bool, args: &[S]) -> (String, String) {
        let output = process::Command::new(BIN_EXE)
            .args(args)
            .env(paths::BAK9_HOME, PathBuf::from(test.fixture_dir()).join(test_common::FIXTURE_USER_HOME))
            .env(test_common::BAK9_TMP_DIR, test.temp_dir())
            .env(test_common::BAK9_TESTS_DIR, test.fixture_dir())
            .output()
            .unwrap();

        assert_eq!(success, output.status.success());
        (String::from_utf8(output.stdout).unwrap(), String::from_utf8(output.stderr).unwrap())
    }

    #[named]
    #[test]
    fn test_help() {
        let test = TESTING.test(function_name!())
            .inherit_fixture_dir()
            .using_temp_dir()
            .build();

        let (stdout, stderr) = cmd(&test, true, &["backup", "--help"]);
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
            .inherit_fixture_dir()
            .using_temp_dir()
            .setup(setup_backup_dir)
            .build();

        let (stdout, stderr) = cmd(&test, false, &["backup", "scheduled"]);
        assert_eq!("", stdout);
        assert_eq!("", stderr);
    }
}