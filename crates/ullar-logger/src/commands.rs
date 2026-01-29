use std::fs::File;
use std::io::Write;
use std::{
    fs::{self, OpenOptions},
    path::Path,
    process::Command,
};

use colored::Colorize;

pub fn log_commands(cmd: &Command) {
    let msg = "\nRunning command:";
    log::info!("{}", msg.bold());
    let command_str = format!("{:?}", cmd).replace("\"", "");
    log::info!("{}\n", command_str.italic());
}

pub fn get_file_cmd_logger(
    file_path: &Path,
    cmd: &Command,
    title: &str,
) -> Result<File, Box<dyn std::error::Error>> {
    if let Some(dir) = file_path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path.with_extension("log"))?;
    writeln!(log, "\n=== {} ===", title)?;
    writeln!(log, "Timestamp: {}", chrono::Local::now())?;
    writeln!(log, "Running command: {:?}", cmd)?;
    Ok(log)
}
