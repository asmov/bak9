use std::{io::Write, path::Path};
use colored::Colorize;
use crate::{error::*, config::*, cli::*, paths::*};

pub(crate) fn run_config(cli: &Cli, subcmd: &ConfigCommand) -> Result<bool> {
    let config_path = select_config_path(&cli)?;
    match subcmd {
        ConfigCommand::Setup => run_config_setup(&config_path, cli.force),
        ConfigCommand::Edit => run_config_edit(&config_path),
        ConfigCommand::Verify => run_config_verify(&config_path, false),
        ConfigCommand::Show => run_config_show(&config_path),
    }
}

fn run_config_setup(config_path: &Path, force: bool) -> Result<bool> {
    if config_path.exists() {
        println!("Verifying config file: {}", config_path.to_str().unwrap().cyan());
        return run_config_verify(config_path, true);
    }

    if !force {
        println!("{} Config file not found: {}", "warning:".yellow(), config_path.to_str().unwrap().cyan());
        if !confirm("Would you like to create it now?")? {
            return Ok(false);
        }
    }   

    let config_dir = config_path.parent()
        .expect("Failed to get parent directory");
    std::fs::create_dir_all(config_dir)
        .map_err(|e| Error::file_io(config_dir, e))?;
    std::fs::write(&config_path, CONFIG_DEFAULTS)
        .map_err(|e| Error::file_io(&config_path, e))?;

    println!("Config file created: {}", config_path.to_str().unwrap().cyan());
    println!("Edit your config with {}\nValidate your config with {}",
        "bak9 config edit".yellow(), "bak9 config verify".yellow());

    if confirm("Would you like to edit it now?")? {
        run_config_edit(config_path)
    } else {
        Ok(true)
    }
}

fn confirm(question: &str) -> Result<bool> {
    print!("{} {question} {} ", "confirm:".bright_yellow(), "[y/N]:".magenta());

    std::io::stdout().flush()
        .expect("Failed to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        _ => Ok(false),
    }
}

fn run_config_edit(config_path: &Path) -> Result<bool> {
    println!("Launching editor for config file: {}", config_path.to_str().unwrap().cyan());

    let output = bak9_os::run_best_editor(config_path, false)
        .map_err(|e| Error::Generic(format!("Failed to run editor :: {e}")))?
        .unwrap_output();

    if !output.status.success() {
        return Err(Error::Generic(format!("Failed to run editor :: {}", String::from_utf8(output.stderr).unwrap())));
    }

    println!("Verifying edit");
    run_config_verify(config_path, true)
}

/// Checks whether config_path exists and prints an error message if it does not.   
/// Returns true if an error message was printed, false otherwise.
fn handle_config_file_not_found(config_path: &Path) -> bool {
    if !config_path.exists() {
        eprintln!("{} Config file not found: {}\n       Run {} to create it.",
            "error:".red(),
            config_path.to_str().unwrap().cyan(),
            "bak9 config setup".yellow());
        true
    } else {
        false
    }
}

/// Returns the config file if it exists and is valid, otherwise returns None if the error was handled.
fn verify_config_file(config_path: &Path) -> Result<Option<BackupConfig>> {
    if handle_config_file_not_found(config_path) {
        return Ok(None);
    }

    let config = match read_config(Some(config_path)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{} Config is invalid :: {e}", "error:".red());
            return Ok(None);
        }
    };

    Ok(Some(config))
}


fn run_config_verify(config_path: &Path, fix: bool) -> Result<bool> {
    let config = match verify_config_file(config_path)? {
        Some(config) => config,
        None => return Ok(false),
    };


    if let Err(e) = verify_backup_dirs(&config) {
        let problem = if fix { "error:".red() } else { "warning:".yellow() };
        eprintln!("{problem} Backup environment is invalid:\n  {}", e.to_string().replace(" :: ", "\n  "));

        if !fix {
            eprintln!("  You may run {} to fix this.", "bak9 config setup".yellow());
            if confirm("Would you like to create necessary directories now?")? {
                println!("Creating directories ...");
                setup_backup_storage_dir(&config.backup_storage_dir_path())?;
                println!("Re-verifying ...");
                run_config_verify(config_path, true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    } else {
        println!("{} Backup environment is valid", "good:".green());
        Ok(true)
    }
}

fn run_config_show(config_path: &Path) -> Result<bool> {
    if handle_config_file_not_found(config_path) {
        return Ok(false);
    }

    let header = format!("bak9 config: {}", config_path.to_string_lossy()); 
    println!("{}", header.cyan());
    println!("{:=<1$}", "".cyan(), header.chars().count());
    print!("{}", std::fs::read_to_string(config_path)
        .map_err(|e| Error::Generic(format!("Unable to read from config file: {} :: {e}", config_path.to_string_lossy().cyan())))?);

    Ok(true)
}

 