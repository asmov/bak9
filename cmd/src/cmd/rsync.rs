use std::path::Path;

const RSYNC_CMD: &'static str = "rsync";
const RSYNC_FLAG_ARCHIVE: &'static str = "--archive";
/// Hard links files from LINKSRC to DEST when they are identical to their counterpart in SOURCE, rather than copying
const RSYNC_FLAG_LINK_DESTINATION: &'static str = "--link-dest";
/// Does not place a file in DEST that no longer exists in SOURCE, regardless of whether it exists in LINKSRC
const RSYNC_FLAG_DELETE: &'static str = "--delete";

pub fn cmd_rsync_full(source_dir: &Path, dest_dir: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(RSYNC_CMD);
    cmd.args(&[
        RSYNC_FLAG_ARCHIVE,
        source_dir.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
    ]);

    cmd
}

pub fn cmd_rsync_incremental(previous_backup_dir: &Path, source_dir: &Path, dest_dir: &Path) -> std::process::Command {
    let hardlink_source = format!("{}/", previous_backup_dir.to_str().unwrap());

    let mut cmd = std::process::Command::new(RSYNC_CMD);
    cmd.args(&[
        RSYNC_FLAG_ARCHIVE,
        RSYNC_FLAG_DELETE,
        RSYNC_FLAG_LINK_DESTINATION,
        &hardlink_source,
        source_dir.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
    ]);

    cmd
}

