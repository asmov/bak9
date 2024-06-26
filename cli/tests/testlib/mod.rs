#![allow(dead_code)]

use std::{sync::OnceLock, path::PathBuf};
use asmov_testing::{self as testing, prelude::*};

pub(crate) const TESTLIB: &'static str = "testlib";
pub(crate) const TESTUSR: &'static str = "testusr";
pub(crate) const SOURCE_PREFIX: &'static str = "source-";
pub(crate) const HOME_TESTUSR: &'static str = "home/testusr";
pub(crate) const MOCK_FS_DIRNAME: &'static str = "mock-fs";
/// The test-run's temporary directory
pub(crate) const ENV_BAK9_TEST_TMP_DIR: &'static str = "BAK9_TEST_TMP_DIR";
/// Where a mock filesystem is located
pub(crate) const ENV_BAK9_TEST_SOURCE_ROOT: &'static str = "BAK9_TEST_SOURCE_ROOT";

pub(crate) fn source_dir(source_num: u8, test: &testing::Test) -> PathBuf {
    test.imported_fixture_dir(testlib_namepath())
        .join(MOCK_FS_DIRNAME)
        .join(format!("{}{source_num}", SOURCE_PREFIX))
        .join(HOME_TESTUSR)
        .canonicalize().unwrap()
}

pub(crate) fn testlib_namepath() -> &'static testing::Namepath {
    static NAMEPATH: OnceLock<testing::Namepath> = OnceLock::new();
    &NAMEPATH.get_or_init(|| testing::Namepath::module(testing::UseCase::Integration, TESTLIB.to_string()))
}

pub(crate) trait TestlibModuleBuilder {
    fn testlib_module_defaults(self) -> Self;
}

impl<'func> TestlibModuleBuilder for testing::ModuleBuilder<'func> {
    fn testlib_module_defaults(self) -> Self {
        bak9::log::Log::init(None, None);
        self.import_fixture_dir(testlib_namepath())
            .base_temp_dir(env!("CARGO_TARGET_TMPDIR"))
    }
}