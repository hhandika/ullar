use std::process::Command;

use super::{re_capture_version, DepMetadata};
use crate::version;

pub const FASTP_EXE: &str = "fastp";

#[derive(Debug, Default)]
pub struct FastpMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
}

impl<'a> FastpMetadata<'a> {
    pub fn new(args: Option<&'a str>) -> Self {
        Self {
            version: version!(FASTP_EXE),
            override_args: args,
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        match &self.version {
            Some(version) => self.metadata(version),
            None => None,
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let executable = FASTP_EXE.to_string();
        let version = re_capture_version(version_data);
        Some(DepMetadata {
            name: "fastp".to_string(),
            version: version.to_string(),
            executable,
            override_args: self.override_args.map(|s| s.to_string()),
        })
    }
}
