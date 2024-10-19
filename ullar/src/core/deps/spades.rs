use std::process::Command;

use crate::version;

use super::{re_capture_version, DepMetadata};

pub const SPADES_EXE: &str = "spades.py";

pub struct SpadesMetadata {
    version: Option<String>,
}

impl Default for SpadesMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl SpadesMetadata {
    pub fn new() -> Self {
        Self {
            version: version!(SPADES_EXE),
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        match &self.version {
            Some(version) => self.metadata(version),
            None => None,
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let executable = SPADES_EXE.to_string();
        let version = re_capture_version(version_data);
        Some(DepMetadata {
            name: "SPAdes".to_string(),
            version: version.to_string(),
            executable,
        })
    }
}
