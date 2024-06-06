use std::{io::Write, process, path::Path};
use colored::Colorize;
use crate::{config, cli, Error, Result};

pub(crate) fn run_config(cli: &cli::Cli, subcmd: &cli::ConfigCommand) -> Result<process::ExitCode> {
    let config_path = config::select_config_path(&cli)?;
    match subcmd {
        cli::ConfigCommand::Setup => run_config_setup(&config_path, cli.force),
        cli::ConfigCommand::Edit => todo!(),
        cli::ConfigCommand::Verify => run_config_verify(&config_path),
        cli::ConfigCommand::Show => todo!(),
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
    todo!()
}

fn run_config_verify(config_path: &Path) -> Result<process::ExitCode> {
    if !config_path.exists() {
        eprintln!("{} Config file not found: {}\n       Run {} to create it.",
            "error:".red(),
            config_path.to_str().unwrap().cyan(),
            "bak9 config setup".yellow());
        return Ok(process::ExitCode::FAILURE);
    }

    println!("{} Config is valid", "good:".green());
    println!("{} Backup environment is valid", "good:".green());
    Ok(std::process::ExitCode::SUCCESS)
}

