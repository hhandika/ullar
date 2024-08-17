//! Utilities for managing dependencies.

use colored::Colorize;
use fastp::FastpMetadata;
use iqtree::IqtreeMetadata;
use mafft::MafftMetadata;
use serde::{Deserialize, Serialize};
use spades::SpadesMetadata;

pub mod fastp;
pub mod iqtree;
pub mod lastz;
pub mod mafft;
pub mod spades;

#[macro_export]
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
    Lastz,
    Mafft,
    Iqtree,
}

const DEPENDENCY_LIST: [Deps; 5] = [
    Deps::Fastp,
    Deps::Spades,
    Deps::Iqtree,
    Deps::Lastz,
    Deps::Mafft,
];

pub struct DependencyCheck {
    pub spades: Option<DepMetadata>,
    pub fastp: Option<DepMetadata>,
    pub lastz: Option<DepMetadata>,
    pub mafft: Option<DepMetadata>,
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
            lastz: None,
            mafft: None,
            iqtree: None,
        }
    }

    pub fn check(&mut self) {
        self.get();
        self.check_spades();
        self.check_fastp();
        self.check_iqtree();
        self.check_mafft();
    }

    fn get(&mut self) {
        DEPENDENCY_LIST.iter().for_each(|dep| match dep {
            Deps::Spades => self.spades(),
            Deps::Fastp => self.fastp(),
            Deps::Lastz => self.lastz(),
            Deps::Iqtree => self.iqtree(),
            Deps::Mafft => self.mafft(),
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

    fn check_mafft(&self) {
        match &self.mafft {
            Some(mafft) => self.print_ok(&mafft.name, &mafft.version),
            None => self.print_error("MAFFT"),
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

    fn mafft(&mut self) {
        let mafft = MafftMetadata::new().get();
        self.mafft = mafft.metadata;
    }

    fn lastz(&mut self) {
        let lastz = lastz::LastzMetadata::new().get();
        self.lastz = lastz.metadata;
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DepMetadata {
    pub name: String,
    pub version: String,
    pub executable: String,
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
