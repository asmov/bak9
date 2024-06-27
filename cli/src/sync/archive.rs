use std::path::PathBuf;
use crate::{error::*, log::*, job::*, backup::*};
use super::Remote;

#[derive(Debug)]
pub struct SyncArchiveJob {
    pub(crate) backup_run_name: BackupRunName,
    pub(crate) remote: Remote,
    pub(crate) source_filepath: PathBuf,
    pub(crate) remote_dest_filepath: PathBuf
}

impl JobTrait for SyncArchiveJob {
    type Output = SyncArchiveJobOutput;

    fn run(&self) -> Result<JobOutput> {
        log_info!("Began uploading archive of {} to remote {}",
            self.backup_run_name.backup_name.tik_name(), self.remote.name.tik_name());

        log_info!("Completed uploading archive of {} to remote {}",
            self.backup_run_name.backup_name.tik_name(), self.remote.name.tik_name());

        Ok(JobOutput::SyncArchive(SyncArchiveJobOutput {
            backup_run_name: self.backup_run_name.clone(),
            remote: self.remote.clone(),
            source_filepath: self.source_filepath.clone(),
            remote_dest_filepath: self.remote_dest_filepath.clone()
        }))
    }
}

#[derive(Debug)]
pub struct SyncArchiveJobOutput {
    pub backup_run_name: BackupRunName,
    pub remote: Remote,
    pub source_filepath: PathBuf,
    pub remote_dest_filepath: PathBuf
}

impl JobOutputTrait for SyncArchiveJobOutput {}


