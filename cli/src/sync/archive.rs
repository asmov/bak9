use crate::{error::*, log::*, job::*};

#[derive(Debug)]
pub struct SyncArchiveJob {

}

impl JobTrait for SyncArchiveJob {
    type Output = SyncArchiveJobOutput;

    fn run(&self) -> Result<JobOutput> {
        Ok(JobOutput::SyncArchive(SyncArchiveJobOutput {}))
    }
    
}

#[derive(Debug)]
pub struct SyncArchiveJobOutput {

}

impl JobOutputTrait for SyncArchiveJobOutput {}


