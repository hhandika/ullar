use std::process::Command;

use crate::version;

use super::{re_capture_version, DepMetadata};

/// Lastz executable.
pub const LASTZ_EXE: &str = "lastz";

pub struct LastzMetadata {
    pub metadata: Option<DepMetadata>,
}

impl Default for LastzMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl LastzMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Self {
        let version: Option<String> = self.get_lastz();
        match version {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
        }
    }

    fn get_lastz(&self) -> Option<String> {
        version!(LASTZ_EXE)
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        Some(DepMetadata {
            name: "LASTZ".to_string(),
            version: version.to_string(),
            executable: LASTZ_EXE.to_string(),
        })
    }
}
