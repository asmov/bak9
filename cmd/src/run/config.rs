use std::{io::Write, process, path::Path};
use colored::Colorize;
use crate::{config, cli, Error, Result};

pub(crate) fn run_config(cli: &cli::Cli, subcmd: &cli::ConfigCommand) -> Result<process::ExitCode> {
    let config_path = config::select_config_path(&cli)?;
    match subcmd {
        cli::ConfigCommand::Setup => run_config_setup(&config_path, cli.force),
        cli::ConfigCommand::Edit => run_config_edit(&config_path),
        cli::ConfigCommand::Verify => run_config_verify(&config_path),
        cli::ConfigCommand::Show => run_config_show(&config_path),
    }
}

fn run_config_setup(config_path: &Path, force: bool) -> Result<process::ExitCode> {
    if config_path.exists() {
        println!("Verifying config file: {}", config_path.to_str().unwrap().cyan());
        return run_config_verify(config_path);
    }

    if !force {
        println!("{} Config file not found: {}", "warning:".yellow(), config_path.to_str().unwrap().cyan());
        print!("{} Would you like to create it now? {} ", "confirm:".bright_yellow(), "[y/N]:".magenta());

        std::io::stdout().flush()
            .expect("Failed to flush stdout");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .expect("Failed to read input");

        if input.trim().to_lowercase() != "y" {
            return Ok(std::process::ExitCode::FAILURE);
        }
    }   

    let config_dir = config_path.parent()
        .expect("Failed to get parent directory");
    std::fs::create_dir_all(config_dir)
        .map_err(|e| Error::new_file_io(config_dir, e))?;
    std::fs::write(&config_path, config::CONFIG_DEFAULTS)
        .map_err(|e| Error::new_file_io(&config_path, e))?;

    println!("Config file created: {}", config_path.to_str().unwrap().cyan());
    println!("Edit your config with {}\nValidate your config with {}", "bak9 config edit".yellow(), "bak9 config verify".yellow());

    print!("{} Would you like to edit it now? {} ", "confirm:".bright_yellow(), "[y/N]:".magenta());

    std::io::stdout().flush()
        .expect("Failed to flush stdout");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
        .expect("Failed to read input");

    if input.trim().to_lowercase() != "y" {
        run_config_edit(config_path)
    } else {
        Ok(std::process::ExitCode::SUCCESS)
    }
}

fn run_config_edit(config_path: &Path) -> Result<process::ExitCode> {
    println!("Launching editor for config file: {}", config_path.to_str().unwrap().cyan());

    let output = bak9_os::run_best_editor(config_path, false)
        .map_err(|e| Error::Generic(format!("Failed to run editor :: {e}")))?
        .unwrap_output();

    if !output.status.success() {
        return Err(Error::Generic(format!("Failed to run editor :: {}", String::from_utf8(output.stderr).unwrap())));
    }

    println!("Verifying edit");
    run_config_verify(config_path)
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

fn run_config_verify(config_path: &Path) -> Result<process::ExitCode> {
    if handle_config_file_not_found(config_path) {
        return Ok(std::process::ExitCode::FAILURE);
    }

    println!("{} Config is valid", "good:".green());
    println!("{} Backup environment is valid", "good:".green());
    Ok(std::process::ExitCode::SUCCESS)
}

fn run_config_show(config_path: &Path) -> Result<process::ExitCode> {
    if handle_config_file_not_found(config_path) {
        return Ok(std::process::ExitCode::FAILURE);
    }

    let header = format!("bak9 config: {}", config_path.to_string_lossy()); 
    println!("{}", header.cyan());
    println!("{:=<1$}", "".cyan(), header.chars().count());
    print!("{}", std::fs::read_to_string(config_path)
        .map_err(|e| Error::Generic(format!("Unable to read from config file: {} :: {e}", config_path.to_string_lossy().cyan())))?);

    Ok(std::process::ExitCode::SUCCESS)
}

 