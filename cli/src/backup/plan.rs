use super::*;
use crate::paths::*;

impl BackupJob {
    /// Creates a backup run plan for a config item, consisting of a series of tasks to be performed in sequence.
    /// May include a full or incremental backup, archiving (.tar.xz), and syncing of backups and archives to remotes.
    pub fn plan(
        backup_type: BackupType,
        cfg_backup: &BackupConfigBackup,
        config: &BackupConfig
    ) -> Result<JobQueueEntry> {
        let bak9_storage_dir = config.bak9_storage_dir();
        let run_name = BackupRunName::new(datetime_now(), hostname(), username(), &cfg_backup.name);
        let source_dir = cfg_backup.source_dir_path();
        let dest_dir = Bak9Path::backup(&bak9_storage_dir, backup_type, &run_name);
        let incremental_source_dir = if backup_type == BackupType::Incremental {
            Some(find_last_backup(
                BackupType::Full,
                hostname(),
                username(),
                &cfg_backup.name,
                &config.backup_storage_dir_path()
            ).unwrap())
        } else {
            None
        };

        let archive_source_dir = &dest_dir;
        let archive_dest_filepath = Bak9Path::archive(&bak9_storage_dir, &run_name);
        let archive_run_name = &run_name;

        let mut series = vec![
            JobQueueEntry::Job {
                job: Job::Backup(BackupJob {
                    backup_type,
                    run_name: run_name.clone(),
                    source_dir: source_dir.clone(),
                    incremental_source_dir: incremental_source_dir.clone(),
                    dest_dir: dest_dir.to_path_buf(),
                }),
                status: JobStatus::Ready,
                result: None
        }];

        if backup_type == BackupType::Full {
            series.push(JobQueueEntry::Job {
                job: Job::Archive(ArchiveJob {
                    backup_run_name: archive_run_name.clone(),
                    source_dir: archive_source_dir.to_path_buf(),
                    dest_filepath: archive_dest_filepath.to_path_buf(),
                }),
                status: JobStatus::Ready,
                result: None
            });
        }

        for cfg_sync in &cfg_backup.syncs {
            for cfg_remote in cfg_sync.remotes(config) {
                match backup_type {
                    BackupType::Full => {
                        if cfg_sync.sync_full {
                            series.push(JobQueueEntry::Job {
                                job: Job::SyncBackup(SyncBackupJob {
                                    remote: Remote {
                                        name: cfg_remote.name.clone(),
                                        host: cfg_remote.host.clone(),
                                        user: cfg_remote.user.clone(),
                                    },
                                    backup_type: backup_type,
                                    backup_run_name: run_name.clone(),
                                    source_dir: source_dir.clone(),
                                    remote_incremental_source_dir: None,
                                    remote_dest_dir: PathBuf::from(&cfg_remote.backup_storage_dir)
                                        .join(backup_type.subdir_name())
                                        .join(&run_name),

                                }),
                                status: JobStatus::Ready,
                                result: None
                            })
                        }
                        if cfg_sync.sync_archive {
                            series.push(JobQueueEntry::Job {
                                job: Job::SyncArchive(SyncArchiveJob {
                                    remote: Remote {
                                        name: cfg_remote.name.clone(),
                                        host: cfg_remote.host.clone(),
                                        user: cfg_remote.user.clone(),
                                    },
                                    backup_run_name: archive_run_name.clone(),
                                    source_filepath: archive_dest_filepath.to_path_buf(),
                                    remote_dest_filepath: PathBuf::from(&cfg_remote.backup_storage_dir)
                                        .join(paths::BACKUP_ARCHIVE_DIRNAME)
                                        .join(&archive_run_name)
                                        .with_extension(paths::TAR_XZ_EXTENSION),
                                }),
                                status: JobStatus::Ready,
                                result: None
                            });
                        }
                    },
                    BackupType::Incremental if cfg_sync.sync_incremental => {
                        let incremental_source_dirname = incremental_source_dir.as_ref().unwrap()
                            .file_name().unwrap().to_str().unwrap();

                        series.push(JobQueueEntry::Job {
                            job: Job::SyncBackup(SyncBackupJob {
                                remote: Remote {
                                    name: cfg_remote.name.clone(),
                                    host: cfg_remote.host.clone(),
                                    user: cfg_remote.user.clone(),
                                },
                                backup_type: backup_type,
                                backup_run_name: run_name.clone(),
                                source_dir: source_dir.clone(),
                                remote_incremental_source_dir: Some(PathBuf::from(&cfg_remote.backup_storage_dir)
                                    .join(backup_type.subdir_name())
                                    .join(incremental_source_dirname)),
                                remote_dest_dir: PathBuf::from(&cfg_remote.backup_storage_dir)
                                    .join(backup_type.subdir_name())
                                    .join(&run_name),
                            }),
                            status: JobStatus::Ready,
                            result: None
                        });
                    },
                    _ => {}
                }
            }
        }

        Ok(JobQueueEntry::Series(series))
    }
}
