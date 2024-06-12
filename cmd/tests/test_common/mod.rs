#![allow(dead_code)]

pub(crate) const FIXTURE_USER_HOME: &'static str = "home/testusr";
pub(crate) const BAK9_TMP_DIR: &'static str = "BAK9_TMP_DIR";
pub(crate) const BAK9_TESTS_DIR: &'static str = "BAK9_TESTS_DIR";

use asmov_testing as testing;
use std::path::PathBuf;
use bak9::paths;

pub(crate) static COMMON_TESTING: testing::StaticModule = testing::module(|| {
    testing::integration("common")
        .base_temp_dir(env!("CARGO_TARGET_TMPDIR"))
        .using_temp_dir()
        .using_fixture_dir()
        .setup(|module_test| {
            std::env::set_var(paths::BAK9_HOME,
                PathBuf::from(module_test.fixture_dir())
                    .join(FIXTURE_USER_HOME)
            );

            std::env::set_var(BAK9_TMP_DIR, PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
            std::env::set_var(BAK9_TESTS_DIR, PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"));
        })
        .build()
});

