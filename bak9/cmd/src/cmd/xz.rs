use std::path::Path;

// tar cf "$home_backup_zip" --use-compress-program='xz -T0' "$home_backup_name"
pub fn cmd_tar_xz(source_dir: &Path, zip_file: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("tar");
    cmd
        .current_dir(source_dir.parent().unwrap())
        .args(&[
            "cf",
            zip_file.to_str().unwrap(),
            "--use-compress-program",
            "xz -T0",
            source_dir.file_name().unwrap().to_str().unwrap(),
        ]);

    cmd
}

