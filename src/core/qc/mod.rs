//! Clean raw read files using Fastp
pub mod fastp;

use std::path::Path;

use crate::core::qc::fastp::FastpRunner;

use super::new::configs::RawReadConfig;

pub struct ReadCleaner<'a> {
    /// Path to the raw read configuration file
    pub config_path: &'a Path,
    /// Should the SHA256 checksum be checked
    /// before cleaning the files
    pub check_sha256: bool,
}

impl ReadCleaner<'_> {
    /// Initialize a new ReadCleaner instance
    pub fn new<'a>(config_path: &'a Path, check_sha256: bool) -> ReadCleaner<'a> {
        ReadCleaner {
            config_path,
            check_sha256,
        }
    }

    /// Clean raw read files using Fastp
    pub fn clean(&self) {}
}
