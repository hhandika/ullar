use std::path::{Path, PathBuf};

use segul::helper::logger;

const LOG_EXTENSION: &str = "log";

pub fn init_logger(dir: &Path, prefix: &str) -> PathBuf {
    let file_path = dir.join(format!("{}.{}", prefix, LOG_EXTENSION));
    logger::init_file_logger(&file_path).expect("Failed to initialize logger");
    file_path
}
