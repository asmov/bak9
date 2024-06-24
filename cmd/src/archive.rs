use std::path::PathBuf;
use crate::{error::*, job::*, backup::*, log::*, cmd::xz};

#[derive(Debug)]
pub struct ArchiveJob {
    pub(crate) backup_run_name: BackupRunName,
    pub(crate) source_dir: PathBuf,
    pub(crate) dest_filepath: PathBuf
}

pub struct ArchiveJobOutput {
    pub(crate) backup_run_name: BackupRunName,
    pub(crate) source_dir: PathBuf,
    pub(crate) dest_filepath: PathBuf
}

impl JobTrait for ArchiveJob {
    type Output = ArchiveJobOutput;

    fn run(&self) -> Result<JobOutput> {
        Log::get().info(&format!("Began archiving `{}`", self.backup_run_name.backup_name));

        let mut tar_xz_cmd = xz::cmd_tar_xz(&self.source_dir, &self.dest_filepath);
        let output = tar_xz_cmd.output().unwrap();

        if !output.status.success() {
            return Err(Error::rsync(output));
        }
    
        Log::get().info(&format!("Completed archiving `{}`", self.backup_run_name.backup_name));

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