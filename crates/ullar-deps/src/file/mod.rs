use std::path::PathBuf;

/// Get path to a command line tool dependency.
pub fn get_exe_path(exe_name: &str) -> Option<PathBuf> {
    match which::which(exe_name) {
        Ok(path) => Some(path),
        Err(_) => {
            eprintln!("Warning: Could not find executable '{}'", exe_name);
            None
        }
    }
}
