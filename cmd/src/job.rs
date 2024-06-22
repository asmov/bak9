//! Each [Job] is a unit of work. Each has an associated [JobPlan], [JobInput], a [JobOutput]. A job is initialized with a
//! [JobPlan]. When it is time to task the job out, the [JobPlan] is used to create a [JobInput], which finalizes exactly what
//! the Job will do.

use std::path::PathBuf;
use crate::{backup::*, config::*, error::*, paths::BACKUP_ARCHIVE_DIRNAME};

#[derive(strum::Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Job {
    Backup(BackupJob),
    Archive(ArchiveJob),
    SyncBackup,
    SyncArchive,
}

impl Job {
    pub fn run(&self) -> Result<JobOutput> {
        match self {
            Job::Backup(job) => job.run(),
            Job::Archive(job) => job.run(),
            Job::SyncBackup => todo!(),
            Job::SyncArchive => todo!(),
        }
    }

    pub fn break_on_error(&self) -> bool {
        match self {
            Job::Backup(job) => job.break_on_error(),
            Job::Archive(job) => job.break_on_error(),
            Job::SyncBackup => true,
            Job::SyncArchive => true,
        }
    }
}

pub enum JobOutput {
    Backup(BackupJobOutput),
    Archive(ArchiveJobOutput),
    SyncBackup,
    SyncArchive,
}

pub type JobQueue = Vec<JobQueueEntry>;

pub enum JobQueueEntry {
    Job { job: Job, status: JobStatus, result: Option<Result<JobOutput>> },
    Series(JobQueue),
}

pub trait JobTrait {
    type Output: JobOutputTrait;

    fn break_on_error(&self) -> bool {
        true
    }

    fn run(&self) -> Result<JobOutput>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum JobStatus {
    Planned,
    Ready,
    Running,
    Completed,
    Failed,
    Canceled,
}

pub trait JobOutputTrait {}

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
        todo!()
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


pub fn run_job_entry(
    queue: &mut JobQueue,
    config: &BackupConfig,
) -> Result<()> {
    for entry in queue {
        match entry {
            JobQueueEntry::Job{job, status, result} => {
                let job_result = job.run();

                *status = if job_result.is_ok() {
                    JobStatus::Completed
                } else if job.break_on_error() {
                    return Err(job_result.err().unwrap())
                } else {
                    JobStatus::Failed
                };

                *result = Some(job.run());
            },
            JobQueueEntry::Series(ref mut series) => {
                run_job_entry(series, config)?;
            },
        }
    }

    Ok(())
}
 
pub fn run_jobs(mut queue: JobQueue, config: &BackupConfig) -> JobResults {
    run_job_entry(&mut queue, config)?;
    /*Log::get().info(&format!("Began {job} backup of `{}`", cfg_backup.name));

    let result = match job {
        BackupJob::Full => {
            backup_full(&cfg_backup, &config)?
        },
        BackupJob::Incremental => {
            backup_incremental(&cfg_backup, &config)?
        }
    };

    results.push(result);
    Log::get().info(&format!("Completed {job} backup of `{}`", cfg_backup.name));*/

    Ok(queue)
}










