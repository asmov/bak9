use std::{fs, sync::OnceLock, io::Write};
use chrono::Timelike;
use colored::Colorize;
use crate::{consts, config::*, schedule::*, paths, backup::*, cli::*};

fn make_log_prefix(topic: &str, prefix: Option<&str>, color: colored::Color) -> String {
    let now = chrono::Local::now();
    let prefix = format!("[{h:0>2}:{m:0>2}:{s:0>2} {topic}] {prefix}",
        h = now.hour(),
        m = now.minute(),
        s = now.second(),
        prefix = prefix.unwrap_or(""));

    prefix.color(color).to_string()
}

pub fn bak9_error_log_prefix() -> String {
    make_log_prefix(consts::BAK9, Some("error: "), colored::Color::Red)
}

pub fn bak9_info_log_prefix() -> String {
    make_log_prefix(consts::BAK9, None, colored::Color::Green)
}

pub trait TikPath {
    fn tik_path(&self) -> String;
    fn tikn_path(&self) -> String;
}

impl TikPath for &std::path::Path {
    fn tik_path(&self) -> String {
        self.to_string_lossy().tik_path()
    }

    fn tikn_path(&self) -> String {
        self.to_string_lossy().tikn_path()
    }
}

impl TikPath for std::path::PathBuf {
    fn tik_path(&self) -> String {
        self.to_string_lossy().tik_path()
    }

    fn tikn_path(&self) -> String {
        self.to_string_lossy().tikn_path()
    }
}

pub trait TikColor {
    const ORANGE: colored::Color = colored::Color::TrueColor { r: 255, g: 140, b: 0 };

    fn tik_color(&self, color: colored::Color) -> String;
    fn tikless_color(&self, color: colored::Color) -> String;

    fn tik_name(&self) -> String {
        self.tik_color(colored::Color::BrightCyan)
    }

    fn tikn_name(&self) -> String {
        self.tikless_color(colored::Color::BrightCyan)
    }

    fn tik_path(&self) -> String {
        self.tik_color(colored::Color::Cyan)
    }

    fn tikn_path(&self) -> String {
        self.tikless_color(colored::Color::Cyan)
    }

    fn tikn_prompt(&self) -> String {
        self.tikless_color(colored::Color::Magenta)
    }

    fn tikn_confirm(&self) -> String {
        self.tikless_color(colored::Color::BrightYellow)
    }

    fn tikn_warning(&self) -> String {
        self.tikless_color(colored::Color::Yellow)
    }

    fn tikn_error(&self) -> String {
        self.tikless_color(colored::Color::Red)
    }

    fn tik_cmd(&self) -> String {
        self.tik_color(Self::ORANGE)
    }

    fn tikn_cmd(&self) -> String {
        self.tik_color(Self::ORANGE)
    }

    fn strip_tik(&self) -> String;

    fn strip_color(&self) -> String;
}

impl TikColor for &str {
    fn tik_color(&self, color: colored::Color) -> String {
        format!("``{}``", self.color(color))
    }

    fn tikless_color(&self, color: colored::Color) -> String {
        format!("{}", self.color(color))
    }

    fn strip_tik(&self) -> String {
        self.replace("``", "")
    }

    fn strip_color(&self) -> String {
        strip_ansi_escapes::strip_str(self)
            .replace("``", "`")
    }
}

impl TikColor for String {
    fn tik_color(&self, color: colored::Color) -> String {
        self.as_str().tik_color(color)
    }

    fn tikless_color(&self, color: colored::Color) -> String {
        self.as_str().tikless_color(color)
    }

    fn strip_tik(&self) -> String {
        self.as_str().strip_tik()
    }

    fn strip_color(&self) -> String {
        self.as_str().strip_color()
    }
}

impl TikColor for std::borrow::Cow<'_, str> {
    fn tik_color(&self, color: colored::Color) -> String {
        self.as_ref().tik_color(color)
    }

    fn tikless_color(&self, color: colored::Color) -> String {
        self.as_ref().tikless_color(color)
    }

    fn strip_tik(&self) -> String {
        self.as_ref().strip_tik()
    }

    fn strip_color(&self) -> String {
        self.as_ref().strip_color()
    }
}

pub struct Log {
    path: Option<std::path::PathBuf>,
    quiet: bool
}

static LOG: OnceLock<Log> = OnceLock::new();

impl Log {
    pub fn init(config: Option<&BackupConfig>, cli: Option<&Cli>) {
        LOG.get_or_init(|| Log::new(config, cli));
    }

    pub(crate) fn get() -> &'static Log {
        LOG.get().unwrap()
    }

    fn new(config: Option<&BackupConfig>, cli: Option<&Cli>) -> Self {
        let quiet = match cli {
            Some(cli) => match cli.subcommand {
                Command::Backup(BackupCommand::Scheduled) => true,
                _ => cli.quiet
            },
            None => false
        };

        if let Some(config) = config { 
            let filename = format!("{}__{}__{}.log", datetimestamp_now(), hostname(), username());
            let path = config.backup_storage_dir_path()
                .join(paths::consts::BACKUP_LOGS_DIRNAME)
                .join(filename);

            match std::fs::write(&path, "") {
                Ok(_) => Self { path: Some(path.canonicalize().unwrap()), quiet },
                Err(_) => Self { path: None, quiet }
            }
        } else {
            Self { path: None, quiet }
        }
    }

    pub fn info(&self, msg: &str) {
        let prefix = bak9_info_log_prefix();
        let log_msg = format!("{} {}", prefix, msg);
        self.write(&log_msg);
    }

    pub fn error(&self, msg: &str) {
        let prefix = bak9_error_log_prefix();
        let log_msg = format!("{} {}", prefix, msg);
        self.write(&log_msg);
    }
    
    pub fn line(&self, msg: &str) {
        println!("{}", msg.strip_tik());
    }

    pub fn eline(&self, msg: &str) {
        eprintln!("{}", msg.strip_tik());
    }

    pub fn out(&self, msg: &str) {
        print!("{}", msg.strip_tik());
    }

    fn write(&self, msg: &str) {
        if let Some(path) = &self.path {
            let file = fs::OpenOptions::new()
                .append(true)
                .open(path);

            match file {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file,  "{}", msg.strip_color()) {
                        if !self.quiet {
                            println!("{}", msg.strip_tik());
                        }

                        eprintln!("{} Unable to write to log file :: {}", bak9_error_log_prefix(), e);
                    }
                },
                Err(e) => {
                    if !self.quiet {
                        println!("{}", msg.strip_tik());
                    }

                    eprintln!("{} Unable to write to log file :: {}", bak9_error_log_prefix(), e.to_string().strip_tik());
                }
            }
        }
        
        if !self.quiet {
            println!("{}", msg.strip_tik());
        }
    }
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        Log::get().info(&format!($($arg)*));
    };
}