use std::process::Command;

use colored::Colorize;

use crate::version;

use super::{check_dependency_match, dependency_not_found, re_capture_version, DepMetadata};

/// Lastz executable.
pub const LASTZ_EXE: &str = "lastz";
pub const LASTZ_NAME: &str = "LASTZ";

/// Metadata for LASTZ.
#[derive(Debug, Default)]
pub struct LastzMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
}

impl<'a> LastzMetadata<'a> {
    pub fn new() -> Self {
        Self {
            version: version!(LASTZ_EXE),
            override_args: None,
        }
    }

    pub fn override_args(mut self, override_args: Option<&'a str>) -> Self {
        self.override_args = override_args;
        self
    }

    pub fn get(&self) -> Option<DepMetadata> {
        match &self.version {
            Some(version) => self.metadata(version),
            None => None,
        }
    }

    pub fn update(&self, config_meta: Option<&DepMetadata>) -> DepMetadata {
        let mut update = self.get().unwrap_or_else(|| {
            panic!(
                "{} LASTZ is not found. 
                Please ensure LASTZ is installed and accessible in your PATH",
                "Error:".red()
            )
        });

        match config_meta {
            Some(dep) => {
                check_dependency_match(&update, &dep.version);
                if dep.override_args.is_some() {
                    update.override_args = dep.override_args.clone();
                }
                update
            }
            None => {
                dependency_not_found("LASTZ");
                update
            }
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        let metadata = DepMetadata::new(LASTZ_NAME, &version, Some(LASTZ_EXE))
            .override_args(self.override_args);
        Some(metadata)
    }
}
