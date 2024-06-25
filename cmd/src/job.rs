//! Each [Job] is a unit of work. Each has an associated [JobPlan], [JobInput], a [JobOutput]. A job is initialized with a
//! [JobPlan]. When it is time to task the job out, the [JobPlan] is used to create a [JobInput], which finalizes exactly what
//! the Job will do.

use crate::{backup::*, config::*, error::*};

#[derive(strum::Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Job {
    Backup(crate::backup::BackupJob),
    Archive(crate::archive::ArchiveJob),
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
    Backup(crate::backup::BackupJobOutput),
    Archive(crate::archive::ArchiveJobOutput),
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

pub fn run_jobs(mut queue: JobQueue, config: &BackupConfig) -> JobResults {
    run_job_entry(&mut queue, config)?;

    // Flatten the results. Input and status are no longer needed. Throw non-breaking errors now if any.
    fn flatten_queue_results(queue: JobQueue) -> JobResults {
        let mut outputs: Vec<JobOutput> = Vec::new();
        for entry in queue {
            match entry {
                JobQueueEntry::Job{job:_, status:_, result} => outputs.push(result.unwrap()?),
                JobQueueEntry::Series(series) => outputs.extend(flatten_queue_results(series)?)
            }
        }

        Ok(outputs)
    }

    flatten_queue_results(queue)
}

fn run_job_entry(queue: &mut JobQueue, config: &BackupConfig) -> Result<()> {
    for entry in queue {
        match entry {
            JobQueueEntry::Job{job, status, result} => {
                match job.run() {
                    Ok(job_output) => {
                        *status = JobStatus::Completed;
                        *result = Some(Ok(job_output));
                    },
                    Err(err) => {
                        if job.break_on_error() {
                            return Err(err)
                        } else {
                            *status = JobStatus::Failed;
                            *result = Some(Err(err));
                        }
                    }
                }
            },
            JobQueueEntry::Series(ref mut series) => {
                run_job_entry(series, config)?;
            },
        }
    }

    Ok(())
}
 