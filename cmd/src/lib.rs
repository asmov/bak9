pub mod error;
pub mod config;
pub mod schedule;
pub mod paths;
pub mod cli;
pub mod cmd;
pub mod backup;
pub mod run;

pub use error::{Error, Result};
pub use run::run_default as run;

pub mod strings {
    pub const BAK9: &'static str = "bak9";
}
