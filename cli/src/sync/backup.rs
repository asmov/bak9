use std::path::PathBuf;
use crate::{error::*, log::*, job::*, backup::*, cmd::rsync};
use super::Remote;

#[derive(Debug)]
pub struct SyncBackupJob {
    pub remote: Remote,
    pub backup_type: BackupType,
    pub backup_run_name: BackupRunName,
    pub source_dir: PathBuf,
    pub remote_incremental_source_dir: Option<PathBuf>,
    pub remote_dest_dir: PathBuf,
}

impl JobTrait for SyncBackupJob {
    type Output = SyncBackupJobOutput;

    fn run(&self) -> Result<JobOutput> {
        log_info!("Began uploading {} backup of {} to remote {}",
            self.backup_type, self.backup_run_name.backup_name.tik_name(), self.remote.name.tik_name());

        let mut rsync_cmd = match self.backup_type {
            BackupType::Full => rsync::cmd_rsync_full(&self.source_dir, &self.remote_dest_dir),
            BackupType::Incremental => rsync::cmd_rsync_incremental_ssh(
                &self.source_dir,
                &self.remote.host,
                self.remote.user.as_ref().map(|s| s.as_str()),
                &self.remote_incremental_source_dir.as_ref().unwrap(),
                &self.remote_dest_dir
            ),
        };

        let output = rsync_cmd.output().unwrap();

        if !output.status.success() {
            return Err(Error::rsync(output));
        }
    
        log_info!("Completed uploading {} backup of {} to {}",
            self.backup_type,
            self.backup_run_name.backup_name.tik_name(),
            format!("{}:{}", self.remote.name, self.remote_dest_dir.to_string_lossy()).tik_path());

        Ok(JobOutput::SyncBackup(SyncBackupJobOutput {
            remote: self.remote.clone(),
            source_dir: self.source_dir.clone(),
            remote_incremental_source_dir: self.remote_incremental_source_dir.clone(),
            remote_dest_dir: self.remote_dest_dir.clone()
        }))
    }
}

#[derive(Debug)]
pub struct SyncBackupJobOutput {
    pub remote: Remote,
    pub source_dir: PathBuf,
    pub remote_incremental_source_dir: Option<PathBuf>,
    pub remote_dest_dir: PathBuf,
}

impl JobOutputTrait for SyncBackupJobOutput {}

