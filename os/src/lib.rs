use std::{env, fs, io, path::{Path, PathBuf}};

const E_STR: &str = "Failed to convert to string"; 

pub fn sanitize_cmd_path(path: &Path) -> &str {
    let path = path.to_str().expect(E_STR);
    if cfg!(target_os = "windows") {
        path.trim_start_matches("\\\\?\\")
    } else {
        path
    }
}

pub fn copy_file(source: &Path, dest: &Path) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    match linux_cp(source, dest) {
        Ok(_) => return Ok(()),
        Err(_) => {}, // fallback
    }
        
    fs::copy(source, dest)
        .map(|_| ())
}

#[cfg(target_os = "linux")]
fn linux_cp(source: &Path, dest: &Path) -> io::Result<()> {
    let output = std::process::Command::new("cp")
        .arg("--preserve")
        .arg(source)
        .arg(dest)
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

/// Retrieves the bak9 data directory if possible, otherwise None.
pub fn user_app_data_dir(mkdir: bool, app_subdirs: PathBuf) -> io::Result<PathBuf> {
    #[cfg(target_os = "linux")]
    let os_data_dir = linux_user_app_data_dir(mkdir)?;
    #[cfg(target_os = "windows")]
    let os_data_dir = windows_user_app_data_dir()?;
    #[cfg(target_os = "macos")]
    let os_data_dir = macos_user_app_data_dir()?;
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    return Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS"));

    let dir = os_data_dir.join(app_subdirs);
    if dir.is_dir() {
        dir.canonicalize()
    } else if mkdir {
        fs::create_dir_all(&dir)?;
        dir.canonicalize()
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, format!("User app data directory not found: {}", dir.to_str().unwrap())))
    }
}

#[cfg(target_os = "linux")]
const ENV_XDG_DATA_HOME: &str = "XDG_DATA_HOME";
#[cfg(target_os = "linux")]
const XDG_DATA_HOME_DEFAULT: &str = "$HOME/.local/share";

#[cfg(target_os = "linux")]
fn linux_user_app_data_dir(mkdir: bool) -> io::Result<PathBuf> {
    let mut var = env::var(ENV_XDG_DATA_HOME)
        .unwrap_or_else(|_| XDG_DATA_HOME_DEFAULT.to_string());

    env::vars().for_each(|(k, v)| var = var.replace(&format!("${k}"), &v));

    let dir = PathBuf::from(var);

    if dir.is_dir() {
        dir.canonicalize()
    } else if mkdir {
        fs::create_dir_all(&dir)?;
        dir.canonicalize()
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound,
            format!("Linux user app data directory not found: {}", dir.to_str().unwrap())))
    }
}

#[cfg(target_os = "windows")]
const ENV_LOCAL_APP_DATA: &str = "LocalAppData";

#[cfg(target_os = "windows")]
fn windows_user_app_data_dir() -> io::Result<PathBuf> {
    let dir: PathBuf = env::var(ENV_LOCAL_APP_DATA)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound,
            format!("Windows %LocalAppData% not found")))?
        .into();

    dir.canonicalize()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound,
            format!("Windows user app data directory not found: {}", dir.to_str().unwrap())))
}

#[cfg(target_os = "macos")]
const MACOS_LIBRARY_APP_SUPPORT: &str = "Library/Application Support";
#[cfg(target_os = "macos")]
const HOME: &str = "HOME";

#[cfg(target_os = "macos")]
fn macos_user_app_data_dir() -> io::Result<PathBuf> {
    let dir: PathBuf = env::var(HOME)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound,
            format!("macOS HOME is not set")))?
        .into();

    dir
        .join(MACOS_LIBRARY_APP_SUPPORT)
        .canonicalize()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound,
            format!("macOS user app data directory not found: {}", dir.to_str().unwrap())))

}

//pub fn run_best_editor(file: &Path) -> 


