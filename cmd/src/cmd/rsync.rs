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

pub fn cmd_rsync_full_ssh(source_dir: &Path, host: &str, remote_user: &str, remote_dest_dir: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(RSYNC_CMD);
    let remote_dest_dir = format!("{remote_user}@{host}/{remote_dest_dir}");
    cmd.args(&[
        RSYNC_FLAG_ARCHIVE,
        source_dir.to_str().unwrap(),
        &remote_dest_dir,
    ]);

    cmd
}

pub fn cmd_rsync_incremental(incremental_source_dir: &Path, source_dir: &Path, dest_dir: &Path) -> std::process::Command {
    let hardlink_source = format!("{}/", incremental_source_dir.to_str().unwrap());

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

pub fn cmd_rsync_incremental_ssh(
    source_dir: &Path,
    host: &str,
    remote_user: Option<&str>,
    remote_incremental_source_dir: &Path,
    remote_dest_dir: &Path
) -> std::process::Command {
    let user_str = match remote_user {
        Some(user) => format!("{}@", user),
        None => String::new(),
    };

    let remote_incremental_source = format!("{user_str}{host}:{}/", remote_incremental_source_dir.to_str().unwrap());
    let remote_dest = format!("{user_str}{host}:{}", remote_dest_dir.to_str().unwrap());

    let mut cmd = std::process::Command::new(RSYNC_CMD);
    cmd.args(&[
        RSYNC_FLAG_ARCHIVE,
        RSYNC_FLAG_DELETE,
        RSYNC_FLAG_LINK_DESTINATION,
        &remote_incremental_source,
        source_dir.to_str().unwrap(),
        &remote_dest,
    ]);

    cmd
}

