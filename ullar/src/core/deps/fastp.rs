use std::process::Command;

use crate::version;

use super::{re_capture_version, DepMetadata};

pub const FASTP_EXE: &str = "fastp";

pub struct FastpMetadata {
    pub metadata: Option<DepMetadata>,
}

impl Default for FastpMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl FastpMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        let version: Option<String> = self.get_fastp();
        match version {
            Some(version) => self.metadata(&version),
            None => None,
        }
    }

    fn get_fastp(&self) -> Option<String> {
        version!(FASTP_EXE)
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let executable = FASTP_EXE.to_string();
        let version = re_capture_version(version_data);
        Some(DepMetadata {
            name: "fastp".to_string(),
            version: version.to_string(),
            executable,
        })
    }
}
