use std::process::Command;

use colored::Colorize;

use super::{check_dependency_match, dependency_not_found, re_capture_version, DepMetadata};
use crate::version;

pub const FASTP_EXE: &str = "fastp";

#[derive(Debug, Default)]
pub struct FastpMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
}

impl<'a> FastpMetadata<'a> {
    pub fn new() -> Self {
        Self {
            version: version!(FASTP_EXE),
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
                "{} fastp is not found. 
                Please ensure fastp is installed and accessible in your PATH",
                "Error:".red()
            )
        });

        match config_meta {
            Some(dep) => {
                check_dependency_match(&update, &dep.version);
                if dep.override_args.is_some() {
                    let default_args = "".to_string();
                    let args = dep.override_args.as_ref().unwrap_or(&default_args);
                    update.override_args = Some(args.to_string());
                }

                update
            }
            None => {
                dependency_not_found(FASTP_EXE);
                update
            }
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let executable = FASTP_EXE.to_string();
        let version = re_capture_version(version_data);
        Some(DepMetadata::new(&executable, &version, Some(&executable)))
    }
}
