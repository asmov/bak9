#![allow(dead_code)]

use lazy_static::lazy_static;
use asmov_testing::{self as testing, prelude::*};

pub(crate) const TESTLIB: &'static str = "testlib";
pub(crate) const TESTUSR: &'static str = "testusr";
pub(crate) const HOME_TESTUSR: &'static str = "home/testusr";
pub(crate) const MOCK_FS_DIRNAME: &'static str = "mock-fs";
/// The test-run's temporary directory
pub(crate) const ENV_BAK9_TEST_TMP_DIR: &'static str = "BAK9_TEST_TMP_DIR";
/// Where a mock filesystem is located
pub(crate) const ENV_BAK9_TEST_MOCK_ROOT_DIR: &'static str = "BAK9_TEST_MOCK_ROOT_DIR";

lazy_static!{
    pub(crate) static ref NAMEPATH: testing::Namepath =
        testing::Namepath::module(testing::UseCase::Integration, TESTLIB.to_string());
}

pub(crate) trait TestlibModuleBuilder {
    fn testlib_module_defaults(self) -> Self;
}

impl<'func> TestlibModuleBuilder for testing::ModuleBuilder<'func> {
    fn testlib_module_defaults(self) -> Self {
        self
            .import_fixture_dir(&NAMEPATH)
            .base_temp_dir(env!("CARGO_TARGET_TMPDIR"))
    }
}
