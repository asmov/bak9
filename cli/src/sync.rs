mod backup;
mod archive;

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub host: String,
    pub user: Option<String>,
}

pub use backup::{SyncBackupJob, SyncBackupJobOutput};
pub use archive::{SyncArchiveJob, SyncArchiveJobOutput};