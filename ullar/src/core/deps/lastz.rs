use std::process::Command;

use crate::version;

use super::{re_capture_version, DepMetadata};

/// Lastz executable.
pub const LASTZ_EXE: &str = "lastz";

pub struct LastzMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
}

impl<'a> LastzMetadata<'a> {
    pub fn new(override_args: Option<&'a str>) -> Self {
        Self {
            version: version!(LASTZ_EXE),
            override_args,
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        match &self.version {
            Some(version) => self.metadata(version),
            None => None,
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        Some(DepMetadata {
            name: "LASTZ".to_string(),
            version: version.to_string(),
            executable: LASTZ_EXE.to_string(),
            override_args: self.override_args.map(|s| s.to_string()),
        })
    }
}
