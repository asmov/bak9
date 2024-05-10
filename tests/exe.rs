mod common;

#[cfg(test)]
pub mod tests {
    use std::{path::Path, process};
    use super::common::*;

    const BIN_EXE: &str = env!("CARGO_BIN_EXE_bak");

    fn cmd<S: AsRef<std::ffi::OsStr>>(success: bool, args: &[S]) -> (String, String) {
        let output = process::Command::new(BIN_EXE)
            .args(args)
            .output()
            .unwrap();

        assert_eq!(success, output.status.success());
        (String::from_utf8(output.stdout).unwrap(), String::from_utf8(output.stderr).unwrap())
    }

    #[test]
    fn test_help() {
        let (stdout, stderr) = cmd(true, &["--help"]);
        assert!(stdout.contains("Usage:"));
        assert!(stderr.is_empty());
    }

    const SOURCE_TXT: &str = "source.txt";
    const SOURCE_TXT_BAK: &str = "source.txt.bak";
    const SOURCE_TXT_BAK_0: &str = "source.txt.bak.0";
    const SOURCE_TXT_BAK_1: &str = "source.txt.bak.1";
    const SOURCE_TXT_BAK_2: &str = "source.txt.bak.2";
    const SOURCE_TXT_BAK_3: &str = "source.txt.bak.3";
    const TESTING_CONTENT: &str = "TESTING_CONTENT";

    #[named]
    #[test]
    fn test_cp_rotation() {
        open_tmpdir(function_name!());
        let source_filepath = tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        
        //STEP: Backup source.txt
        //RESULT: source.txt.bak should exist
        let (stdout, stderr) = cmd(true, &[&source_filepath]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(tmpfile_exists(SOURCE_TXT_BAK, function_name!()));

        //STEP: Backup source.txt again, unchanged.
        //RESULT: No new backup should be created
        let (stdout, stderr) = cmd(true, &[&source_filepath]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK, function_name!()));

        //STEP: Append to source.txt and then backup again
        //RESULT: The .bak file should have been moved to .bak.1. A .bak.0 file should be created
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        let (stdout, stderr) = cmd(true, &[&source_filepath]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(tmpfile_exists(SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK_1, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_2, function_name!()));
        assert!(!tmpfile_diff(SOURCE_TXT, SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_diff(SOURCE_TXT, SOURCE_TXT_BAK_1, function_name!()));

        //STEP: Append to source.txt and then backup again.
        //RESULT: Rotation files should now be .bak.0, .bak.1, and .bak.2 
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        let (stdout, stderr) = cmd(true, &[&source_filepath]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(tmpfile_exists(SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK_1, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK_2, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK, function_name!()));
        assert!(!tmpfile_diff(SOURCE_TXT, SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_diff(SOURCE_TXT, SOURCE_TXT_BAK_1, function_name!()));
        assert!(tmpfile_diff(SOURCE_TXT_BAK_1, SOURCE_TXT_BAK_2, function_name!()));

        //STEP: Append to source.txt and then backup again, with NUM=3
        //RESULT: Rotation files should still be .bak.0, .bak.1, .bak.2, and .bak.3
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "-n", "3"]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(tmpfile_exists(SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK_1, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK_2, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_3, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK, function_name!()));
        assert!(!tmpfile_diff(SOURCE_TXT, SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_diff(SOURCE_TXT, SOURCE_TXT_BAK_1, function_name!()));
        assert!(tmpfile_diff(SOURCE_TXT_BAK_1, SOURCE_TXT_BAK_2, function_name!()));

        //STEP: Append to source.txt and then backup again, with NUM=2
        //RESULT: Rotation files should now be .bak.0 and .bak.1
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "-n", "2"]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(tmpfile_exists(SOURCE_TXT_BAK_0, function_name!()));
        assert!(tmpfile_exists(SOURCE_TXT_BAK_1, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_2, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_3, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK, function_name!()));

        //STEP: Append to source.txt and then backup again, with NUM=1
        //RESULT: Only a .bak file should exist
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "-n", "1"]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        assert!(tmpfile_exists(SOURCE_TXT_BAK, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_0, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_1, function_name!()));
        assert!(!tmpfile_exists(SOURCE_TXT_BAK_2, function_name!()));

        close_tmpdir(function_name!());
    }

    fn wipe_source(source_filepath: &Path) {
        cmd(true, &["-qf", source_filepath.to_str().unwrap(), "rm"]);
    }

    #[named]
    #[test]
    fn test_list_and_wipe() {
        open_tmpdir(function_name!());
        let source_filepath = tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        wipe_source(&source_filepath); // just in case, wipe all app data directory files
        
        //PREP: Backup source.txt three times, appending before each.
        cmd(true, &[&source_filepath]);
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        cmd(true, &[&source_filepath]);
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        cmd(true, &[&source_filepath]);

        //STEP: List the (three) backups
        //RESULT: Rotation of .bak.1, .bak.2, .bak.2
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "ls"]);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 4); // line 0 is a header
        assert_eq!(lines[1].trim(), SOURCE_TXT_BAK_0);
        assert_eq!(lines[2].trim(), SOURCE_TXT_BAK_1);
        assert_eq!(lines[3].trim(), SOURCE_TXT_BAK_2);

        //STEP: Create a backup in the user app data directory. List (four) backups
        //RESULT: Three backups in the original directory, 1 in the new directory
        cmd(true, &[source_filepath.to_str().unwrap(), "-"]);
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "ls"]);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 6); // line 0 and 4 are headers
        assert_eq!(lines[1].trim(), SOURCE_TXT_BAK_0);
        assert_eq!(lines[2].trim(), SOURCE_TXT_BAK_1);
        assert_eq!(lines[3].trim(), SOURCE_TXT_BAK_2);
        assert_eq!(lines[5].trim(), SOURCE_TXT_BAK);

        //STEP: Wipe them all. Run a list again.
        //RESULT: List should be empty
        let (stdout, stderr) = cmd(true, &["-qf", source_filepath.to_str().unwrap(), "rm"]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "ls"]);
        assert!(stdout.is_empty(), "stdout: {}", stdout);
        assert!(stderr.is_empty(), "stderr: {}", stderr);

        close_tmpdir(function_name!());
    }

    #[named]
    #[test]
    fn test_diff() {
        open_tmpdir(function_name!());
        let source_filepath = tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        
        //PREP: Backup source.txt twice, appending before each.
        cmd(true, &[&source_filepath]);
        tmpfile_append(TESTING_CONTENT, SOURCE_TXT, function_name!());
        cmd(true, &[&source_filepath]);

        //STEP: Diff between source.txt and .bak.0 
        //RESULT: "No difference"
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "diff", "0"]);
        assert_eq!("No difference", stdout.trim());
        assert!(stderr.is_empty(), "stderr: {}", stderr);

        //STEP: Diff between source.txt and .bak.1 
        //RESULT: Difference it output
        let (stdout, stderr) = cmd(true, &[source_filepath.to_str().unwrap(), "diff", "1"]);
        assert!(stderr.is_empty(), "stderr: {}", stderr);
        // the last line should be either "< TESTING_CONTENT" (diff) or "-TESTING_CONTENT" (git)
        let last_line = stdout.lines().collect::<Vec<&str>>().last().unwrap().trim();
        // use .contains() to (poorly) ignore terminal color codes
        assert!(last_line.contains(&format!("> {}", TESTING_CONTENT)) || last_line.contains(&format!("-{}", TESTING_CONTENT)));

        close_tmpdir(function_name!());
    }
}
