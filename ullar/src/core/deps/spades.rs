use std::process::Command;

use colored::Colorize;

use crate::version;

use super::{check_dependency_match, dependency_not_found, re_capture_version, DepMetadata};

pub const SPADES_EXE: &str = "spades.py";
const SPADES_NAME: &str = "SPAdes";

#[derive(Debug, Default)]
pub struct SpadesMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
}

impl<'a> SpadesMetadata<'a> {
    pub fn new() -> Self {
        Self {
            version: version!(SPADES_EXE),
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
                "{} SPAdes is not found. 
                Please ensure SPAdes is installed and accessible in your PATH",
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
                dependency_not_found(SPADES_NAME);
                update
            }
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let executable = SPADES_EXE.to_string();
        let version = re_capture_version(version_data);
        Some(DepMetadata::new(SPADES_NAME, &version, Some(&executable)))
    }
}
