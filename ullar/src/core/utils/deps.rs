//! Utilities for managing dependencies.

use std::process::Command;

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::core::{assembly::spades::SPADES_EXE, clean::fastp::FASTP_EXE};

const IQTREE2_EXE: &str = "iqtree2";
const IQTREE_EXE: &str = "iqtree";

macro_rules! version {
    ($exe: ident) => {{
        let output = Command::new($exe).arg("--version").output();

        match output {
            Err(_) => None,
            Ok(output) => {
                // Look from stdout first, otherwise stderr
                let version = String::from_utf8_lossy(&output.stdout);
                if version.is_empty() {
                    Some(String::from_utf8_lossy(&output.stderr).to_string())
                } else {
                    Some(version.to_string())
                }
            }
        }
    }};
}

enum Deps {
    Spades,
    Fastp,
    Iqtree,
}

const DEPENDENCY_LIST: [Deps; 3] = [Deps::Fastp, Deps::Spades, Deps::Iqtree];

pub struct DependencyCheck {
    pub spades: Option<DepMetadata>,
    pub fastp: Option<DepMetadata>,
    pub iqtree: Option<DepMetadata>,
}

impl Default for DependencyCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyCheck {
    pub fn new() -> Self {
        Self {
            spades: None,
            fastp: None,
            iqtree: None,
        }
    }

    pub fn check(&mut self) {
        self.get();
        self.check_spades();
        self.check_fastp();
        self.check_iqtree();
    }

    fn get(&mut self) {
        DEPENDENCY_LIST.iter().for_each(|dep| match dep {
            Deps::Spades => self.spades(),
            Deps::Fastp => self.fastp(),
            Deps::Iqtree => self.iqtree(),
        });
    }

    fn check_spades(&self) {
        match &self.spades {
            Some(spades) => self.print_ok(&spades.name, &spades.version),
            None => self.print_error("SPAdes"),
        }
    }

    fn check_fastp(&self) {
        match &self.fastp {
            Some(fastp) => self.print_ok(&fastp.name, &fastp.version),
            None => self.print_error("fastp"),
        }
    }

    fn check_iqtree(&self) {
        match &self.iqtree {
            Some(iqtree) => self.print_ok(&iqtree.name, &iqtree.version),
            None => self.print_error("IQ-TREE"),
        }
    }

    fn print_ok(&self, name: &str, version: &str) {
        let app = format!("{} v{}", name, version);
        log::info!("{:18}: {}", app, "[OK]".green())
    }

    fn print_error(&self, name: &str) {
        log::error!("{:18}: {}", name, "[NOT FOUND]".red())
    }

    fn spades(&mut self) {
        let spades = SpadesMetadata::new().get();
        self.spades = spades.metadata;
    }

    fn fastp(&mut self) {
        let fastp = FastpMetadata::new().get();
        self.fastp = fastp.metadata;
    }

    fn iqtree(&mut self) {
        let iqtree = IqtreeMetadata::new().get();
        self.iqtree = iqtree.metadata;
    }
}

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

    pub fn get(&self) -> Self {
        let version: Option<String> = self.get_spades();
        match version {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
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

    pub fn get(&self) -> Self {
        let version: Option<String> = self.get_fastp();
        match version {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
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

pub struct MafftMetadata {
    pub metadata: Option<DepMetadata>,
}

impl Default for MafftMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl MafftMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Self {
        let version_data: Option<String> = self.get_mafft();
        if version_data.is_none() {
            return Self { metadata: None };
        }

        match version_data {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
        }
    }

    /// Get the version of fastp
    #[cfg(target_family = "windows")]
    fn get_mafft(&self) -> Option<String> {
        let mafft_exe = crate::core::alignment::mafft::MAFFT_WINDOWS;

        let output = Command::new(mafft_exe).arg("-h").output();
        match output {
            Err(_) => None,
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                Some(version.to_string())
            }
        }
    }

    /// Get the version of mafft unix
    #[cfg(target_family = "unix")]
    fn get_mafft(&self) -> Option<String> {
        let mafft_exe = crate::core::alignment::mafft::MAFFT_EXE;

        let output = Command::new(mafft_exe).arg("--version").output();
        match output {
            Err(_) => None,
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                Some(version.to_string())
            }
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = self.capture_version(version_data);
        let executable = "mafft".to_string();
        Some(DepMetadata {
            name: "MAFFT".to_string(),
            version,
            executable,
        })
    }

    fn capture_version(&self, version_data: &str) -> String {
        let re = regex::Regex::new(r"v\d+\.\d+").expect("Failed to compile regex");
        let captures = re.captures(version_data);

        match captures {
            None => "Unknown".to_string(),
            Some(captures) => captures
                .get(0)
                .expect("Failed to get version")
                .as_str()
                .to_string(),
        }
    }
}

pub struct IqtreeMetadata {
    metadata: Option<DepMetadata>,
}

impl Default for IqtreeMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl IqtreeMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Self {
        let version_data: Option<String> = self.get_iqtree();
        if version_data.is_none() {
            return Self { metadata: None };
        }

        match version_data {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
        }
    }

    fn get_iqtree(&self) -> Option<String> {
        let version_1 = version!(IQTREE_EXE);
        let version_2 = version!(IQTREE2_EXE);

        if version_1.is_some() {
            version_1
        } else {
            version_2
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        let executable = self.get_executable(&version);
        let name = self.name(&version);
        Some(DepMetadata {
            name,
            version,
            executable,
        })
    }

    fn get_executable(&self, version: &str) -> String {
        if version.starts_with("2.") {
            IQTREE2_EXE.to_string()
        } else {
            IQTREE_EXE.to_string()
        }
    }

    fn name(&self, version: &str) -> String {
        if version.starts_with("2.") {
            "IQ-TREE 2".to_string()
        } else {
            "IQ-TREE".to_string()
        }
    }
}

fn re_capture_version(version: &str) -> String {
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").expect("Failed to compile regex");
    let captures = re.captures(version);

    match captures {
        None => "Unknown".to_string(),
        Some(captures) => captures
            .get(0)
            .expect("Failed to get version")
            .as_str()
            .to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DepMetadata {
    pub name: String,
    pub version: String,
    pub executable: String,
}
