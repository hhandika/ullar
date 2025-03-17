use colored::Colorize;

#[cfg(target_family = "unix")]
use crate::version;
use std::process::Command;

use super::{check_dependency_match, dependency_not_found, DepMetadata};

/// Default MAFFT executable for Unix systems
#[cfg(target_family = "unix")]
pub const MAFFT_EXE: &str = "mafft";
const MAFFT_NAME: &str = "MAFFT";

/// Default MAFFT for Windows systems
#[cfg(target_family = "windows")]
pub const MAFFT_EXE: &str = "mafft.bat";

#[derive(Debug, Default)]
pub struct MafftMetadata<'a> {
    name: String,
    override_args: Option<&'a str>,
}

impl<'a> MafftMetadata<'a> {
    pub fn new() -> Self {
        Self {
            name: MAFFT_NAME.to_string(),
            override_args: None,
        }
    }

    pub fn override_args(mut self, override_args: Option<&'a str>) -> Self {
        self.override_args = override_args;
        self
    }

    pub fn get(&self) -> Option<DepMetadata> {
        let version_data: Option<String> = self.get_mafft();
        version_data.as_ref().and_then(|v| self.metadata(v))
    }

    pub fn update(&self, config_meta: Option<&DepMetadata>) -> DepMetadata {
        let mut update = self.get().unwrap_or_else(|| {
            panic!(
                "{} MAFFT is not found. 
                Please ensure MAFFT is installed and accessible in your PATH",
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
                dependency_not_found(MAFFT_NAME);
                update
            }
        }
    }

    /// Get the version of fastp
    #[cfg(target_family = "windows")]
    fn get_mafft(&self) -> Option<String> {
        // let output = Command::new("wsl.exe").arg(MAFFT_EXE).arg("-h").output();
        let output = Command::new(MAFFT_EXE).arg("-h").output();
        match output {
            Err(_) => None,
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stderr);
                Some(version.to_string())
            }
        }
    }

    /// Get the version of mafft unix
    #[cfg(target_family = "unix")]
    fn get_mafft(&self) -> Option<String> {
        version!(MAFFT_EXE)
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = self.capture_version(version_data);
        Some(DepMetadata::new(&self.name, &version, Some(MAFFT_EXE)))
    }

    fn capture_version(&self, version_data: &str) -> String {
        let re = regex::Regex::new(r"\d+\.\d+").expect("Failed to compile regex");
        let captures = re.captures(version_data);

        match captures {
            None => "".to_string(),
            Some(captures) => captures
                .get(0)
                .expect("Failed to get version")
                .as_str()
                .to_string(),
        }
    }
}
