use std::path::PathBuf;
use crate::{error::*, job::*, backup::*, log::*, cmd::xz};

#[derive(Debug)]
pub struct ArchiveJob {
    pub(crate) backup_run_name: BackupRunName,
    pub(crate) source_dir: PathBuf,
    pub(crate) dest_filepath: PathBuf
}

pub struct ArchiveJobOutput {
    pub backup_run_name: BackupRunName,
    pub source_dir: PathBuf,
    pub dest_filepath: PathBuf
}

impl JobTrait for ArchiveJob {
    type Output = ArchiveJobOutput;

    fn run(&self) -> Result<JobOutput> {
        log_info!("Began archiving {}", self.backup_run_name.backup_name.tik_name());

        let mut tar_xz_cmd = xz::cmd_tar_xz(&self.source_dir, &self.dest_filepath);
        let output = tar_xz_cmd.output().unwrap();

        if !output.status.success() {
            return Err(Error::rsync(output));
        }
    
        log_info!("Completed archiving {} to {}",
            self.backup_run_name.backup_name.tik_name(), self.dest_filepath.to_str().unwrap().tik_path());

        Ok(JobOutput::Archive(ArchiveJobOutput {
            backup_run_name: self.backup_run_name.clone(),
            source_dir: self.source_dir.clone(),
            dest_filepath: self.dest_filepath.clone()
        }))
    }
}

impl JobOutputTrait for ArchiveJobOutput {}

impl ArchiveJobOutput {
    pub fn new(backup_run_name: BackupRunName, source_dir: PathBuf, dest_filepath: PathBuf) -> Self {
        Self {
            backup_run_name,
            source_dir,
            dest_filepath
        }
    }
}