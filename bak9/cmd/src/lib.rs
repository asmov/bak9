pub mod error;
pub mod config;
pub mod schedule;
pub mod paths;
pub mod cli;
pub mod cmd;
pub mod backup;

#[cfg(test)]
pub mod testing;

pub fn run() -> std::process::ExitCode {
    todo!()
}