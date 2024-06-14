#![allow(dead_code)]

use std::path::PathBuf;
use lazy_static::lazy_static;
use asmov_testing as testing;
use bak9::paths;

pub(crate) const FIXTURE_USER_HOME: &'static str = "home/testusr";
pub(crate) const BAK9_TMP_DIR: &'static str = "BAK9_TMP_DIR";
pub(crate) const BAK9_TESTS_DIR: &'static str = "BAK9_TESTS_DIR";

lazy_static!{
    pub(crate) static ref NAMEPATH: testing::Namepath =
        testing::Namepath::module(testing::UseCase::Integration, "common".to_string());
}

pub(crate) fn setup_env(testable: &mut impl testing::Testable) {
    std::env::set_var(paths::BAK9_HOME,
        PathBuf::from(testable.imported_fixture_dir(&*NAMEPATH))
            .join(FIXTURE_USER_HOME)
    );

    std::env::set_var(BAK9_TMP_DIR, PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    std::env::set_var(BAK9_TESTS_DIR, PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"));
}


