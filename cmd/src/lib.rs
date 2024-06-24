pub mod error;
#[macro_use]
pub mod config;
pub mod schedule;
pub mod paths;
pub mod cli;
pub mod cmd;
pub mod backup;
pub mod run;
pub mod log;
pub mod job;
pub mod archive;

pub use error::{Error, Result};
pub use run::run_main as run;

pub mod strings {
    pub const BAK9: &'static str = "bak9";
}
