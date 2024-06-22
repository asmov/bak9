//! Each [Job] is a unit of work. Each has an associated [JobPlan], [JobInput], a [JobOutput]. A job is initialized with a
//! [JobPlan]. When it is time to task the job out, the [JobPlan] is used to create a [JobInput], which finalizes exactly what
//! the Job will do.

use std::path::PathBuf;
use crate::{backup::*, config::*, error::*, paths::BACKUP_ARCHIVE_DIRNAME};

pub enum Job {
    Backup(BackupJob),
    Archive(ArchiveJob),
    SyncBackup,
    SyncArchive,
}

pub type JobQueue = Vec<JobQueueEntry>;

pub enum JobQueueEntry {
    Job(Job),
    Queue(JobQueue),
}

pub enum Prep<I> {
    Series(JobQueue, usize),
    Input(I)
}

pub trait JobImpl {
    type Plan: JobPlan;
    type Input: JobInput;
    type Output: JobOutput;
    
    fn plan(plan: Self::Plan) -> Self;
    fn prepare(&mut self, prep: Prep<Self::Input>, config: &BackupConfig) -> Result<&Self::Input>;
    fn get_plan(&self) -> &Self::Plan;
    fn input(&self) -> Option<&Self::Input>;
    fn output(&self) -> Option<&Self::Output>;
    fn status(&self) -> JobStatus;
    fn run(&mut self) -> &Self::Output;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Planned,
    Ready,
    Running,
    Completed,
    Failed,
    Canceled,
}

pub trait JobPlan: Sized {}
pub trait JobInput {}
pub trait JobOutput {}

pub struct ArchiveJob {
    plan: ArchiveJobPlan,
    input: Option<ArchiveJobInput>,
    output: Option<ArchiveJobOutput>,
    status: JobStatus,
}

impl JobImpl for ArchiveJob {
    type Plan = ArchiveJobPlan;
    type Input = ArchiveJobInput;
    type Output = ArchiveJobOutput;

    fn plan(plan: Self::Plan) -> Self {
        Self {
            plan: plan,
            input: None,
            output: None,
            status: JobStatus::Planned,
        }
    }

    fn prepare(&mut self, prep: Prep<Self::Input>, config: &BackupConfig) -> Result<&Self::Input> {
        let input = if let Prep::Input(input) = prep {
            input
        } else if let Prep::Series(queue, index) = prep {
            let backup_job = queue.iter()
                .rev()
                .skip(queue.len() - index)
                .find_map(|entry| match entry { JobQueueEntry::Job(Job::Backup(job)) => Some(job), _ => None })
                .expect("Expected a backup job in the series");

            let source_dir = backup_job.output().expect("Some")
                .dest_dir().to_path_buf();

            ArchiveJobInput {
                source_dir
            }
        } else {
            unreachable!()
        };

        self.input = Some(input);
        Ok(self.input.as_ref().expect("Some"))
    }


    fn get_plan(&self) -> &Self::Plan {
        &self.plan
    }

    fn input(&self) -> Option<&Self::Input> {
        self.input.as_ref()
    }

    fn output(&self) -> Option<&Self::Output> {
        self.output.as_ref()
    }

    fn status(&self) -> JobStatus {
        self.status
    }

    fn run(&mut self) -> &Self::Output {
        self.status = JobStatus::Running;

        self.status = JobStatus::Completed;
        self.output = Some(ArchiveJobOutput {});
        self.output.as_ref().expect("Some")
    }
}

pub struct ArchiveJobPlan {
    pub backup_name: String,
}

pub struct ArchiveJobInput {
    source_dir: PathBuf,
}

pub struct ArchiveJobOutput {
    archive_filepath: PathBuf
}

impl JobPlan for ArchiveJobPlan {}
impl JobInput for ArchiveJobInput {}
impl JobOutput for ArchiveJobOutput {}




