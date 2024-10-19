use std::process::Command;

use crate::version;

use super::{re_capture_version, DepMetadata};

pub const SPADES_EXE: &str = "spades.py";

pub struct SpadesMetadata {
    pub metadata: Option<DepMetadata>,
}

impl Default for SpadesMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl SpadesMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        let version: Option<String> = self.get_spades();
        match version {
            Some(v) => self.metadata(&v),
            None => None,
        }
    }

    fn get_spades(&self) -> Option<String> {
        version!(SPADES_EXE)
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
