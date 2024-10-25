//! Utilities for managing dependencies.
use colored::Colorize;
use comfy_table::{Cell, Color, Table};
use fastp::FastpMetadata;
use iqtree::IqtreeMetadata;
use lastz::LastzMetadata;
use mafft::MafftMetadata;
use serde::{Deserialize, Serialize};
use spades::SpadesMetadata;

pub mod fastp;
pub mod iqtree;
pub mod lastz;
pub mod mafft;
pub mod spades;

pub enum Dependency {
    Fastp,
    Spades,
    Lastz,
    Mafft,
    Iqtree,
}

/// Check the version of the given executable
/// If the executable is not found, return None
/// Otherwise, return the version string
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

pub fn check_dependency_match(dep: &DepMetadata, version: &str) {
    if dep.version != version {
        log::warn!(
            "\n{} Version mismatch for {}. Expected: {}, Found: {}",
            "Warning:".yellow(),
            dep.name,
            dep.version,
            version
        );
    }
}

/// Data structure to store dependency metadata
/// Shared by all dependencies
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DepMetadata {
    pub name: String,
    pub version: String,
    pub executable: String,
    /// Additional arguments/flags for the executable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
}

pub struct DependencyCheck {
    fastp: Option<DepMetadata>,
    spades: Option<DepMetadata>,
    lastz: Option<DepMetadata>,
    mafft: Option<DepMetadata>,
    iqtree: Option<DepMetadata>,
}

impl DependencyCheck {
    pub fn new() -> Self {
        Self {
            fastp: FastpMetadata::new(None).get(),
            spades: SpadesMetadata::new(None).get(),
            lastz: LastzMetadata::new(None).get(),
            mafft: MafftMetadata::new(None).get(),
            iqtree: IqtreeMetadata::new(None).get(),
        }
    }

    pub fn with_override_args(override_args: Option<&str>) -> Self {
        Self {
            fastp: FastpMetadata::new(override_args).get(),
            spades: SpadesMetadata::new(override_args).get(),
            lastz: LastzMetadata::new(override_args).get(),
            mafft: MafftMetadata::new(override_args).get(),
            iqtree: IqtreeMetadata::new(override_args).get(),
        }
    }

    pub fn check(&mut self) {
        let mut table = self.log_status();
        self.log_read_cleaning(&mut table);
        self.log_denovo_assembly(&mut table);
        self.log_contig_mapping(&mut table);
        self.log_sequence_alignment(&mut table);
        self.log_phylogenetic_inference(&mut table);
        log::info!("{}", table);
    }

    fn log_status(&self) -> Table {
        log::info!("{}", "Dependencies".cyan());
        let mut table = Table::new();
        table.set_header(["Features", "Dependencies", "Version", "Status"]);
        table
    }

    fn log_read_cleaning(&mut self, table: &mut Table) {
        let feature = "Read cleaning";
        match &self.fastp {
            Some(metadata) => {
                let cells = self.get_cell(feature, "fastp", &metadata.version, true);
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "fastp", "Unknown", false);
                table.add_row(cells);
            }
        }
    }

    fn log_denovo_assembly(&mut self, table: &mut Table) {
        let feature = "De novo assembly";
        match &self.spades {
            Some(metadata) => {
                let cells = self.get_cell(feature, "SPAdes", &metadata.version, true);
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "SPAdes", "Unknown", false);
                table.add_row(cells);
            }
        }
    }

    fn log_contig_mapping(&mut self, table: &mut Table) {
        let feature = "Contig mapping";
        match &self.lastz {
            Some(metadata) => {
                let cells = self.get_cell(feature, "LASTZ", &metadata.version, true);
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "LASTZ", "Unknown", false);
                table.add_row(cells);
            }
        }
    }

    fn log_sequence_alignment(&mut self, table: &mut Table) {
        let feature = "Sequence alignment";
        match &self.mafft {
            Some(metadata) => {
                let cells = self.get_cell(feature, "MAFFT", &metadata.version, true);
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "MAFFT", "Unknown", false);
                table.add_row(cells);
            }
        }
    }

    fn log_phylogenetic_inference(&mut self, table: &mut Table) {
        let feature = "Phylogenetic inference";
        match &self.iqtree {
            Some(metadata) => {
                let cells = self.get_cell(feature, "IQ-TREE", &metadata.version, true);
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "IQ-TREE", "Unknown", false);
                table.add_row(cells);
            }
        }
    }

    fn get_cell(&self, feature: &str, app: &str, version: &str, ok: bool) -> Vec<Cell> {
        let status = self.status_ok(ok);
        vec![
            Cell::new(feature),
            Cell::new(app),
            Cell::new(version),
            status,
        ]
    }

    fn status_ok(&self, ok: bool) -> Cell {
        if ok {
            Cell::new("[OK]").fg(Color::Green)
        } else {
            Cell::new("[ERROR]").fg(Color::Red)
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
