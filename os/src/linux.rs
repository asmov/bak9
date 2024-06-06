use std::{env, path::Path, process};
use anyhow::anyhow;
use crate::{E_STR, CommandReturn};

const ENV_XDG_SESSION_TYPE: &str = "XDG_SESSION_TYPE";
const WHICH: &str = "which";

pub(crate) fn is_gui() -> bool {
    env::var(ENV_XDG_SESSION_TYPE).is_ok()
}

pub(crate) fn run_best_editor(file: &Path, child_process: bool) -> anyhow::Result<CommandReturn> {
    let editor_cmd = if is_gui() {
        xdg_open_command(file)
    } else if let Some(cmd) = editor_command(file) {
        cmd
    } else if let Some(cmd) = guess_editor_command(file) {
        cmd
    } else {
        return Err(anyhow!("No editor found"))
    };

    CommandReturn::run(editor_cmd, child_process)
}

fn xdg_open_command(file: &Path) -> process::Command {
    let mut cmd = process::Command::new("xdg-open");
    cmd.arg(file);
    cmd
}

fn editor_command(file: &Path) -> Option<process::Command> {
    let which_editor = process::Command::new(WHICH)
        .arg("$EDITOR")
        .output()
        .ok()?;

    if !which_editor.status.success() || which_editor.stdout.is_empty() {
        return None;
    }
    
    let editor_path = String::from_utf8(which_editor.stdout).expect(E_STR);
    let mut cmd = process::Command::new(editor_path.trim());
    cmd.arg(file);
    Some(cmd)
}

const EDITOR_GUESSES: [&str; 4] = ["nvim", "vim", "vi", "nano"];

fn guess_editor_command(file: &Path) -> Option<process::Command> {
    for guess in EDITOR_GUESSES {
        let which_guess = process::Command::new(WHICH)
            .arg(guess)
            .output();

        let which_guess = match which_guess {
            Ok(which_guess) => which_guess,
            Err(_) => continue,
        };

        if !which_guess.status.success() || which_guess.stdout.is_empty() {
            continue;
        }
        
        let editor_path = String::from_utf8(which_guess.stdout).expect(E_STR);
        let mut cmd = process::Command::new(editor_path.trim());
        cmd.arg(file);
        return Some(cmd)
    }
    
    None
}