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

pub struct DependencyCheck {
    fastp: FastpMetadata,
    spades: SpadesMetadata,
    lastz: LastzMetadata,
    mafft: MafftMetadata,
    iqtree: IqtreeMetadata,
}

impl Default for DependencyCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyCheck {
    pub fn new() -> Self {
        Self {
            fastp: FastpMetadata::new(),
            spades: SpadesMetadata::new(),
            lastz: LastzMetadata::new(),
            mafft: MafftMetadata::new(),
            iqtree: IqtreeMetadata::new(),
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
        self.fastp = FastpMetadata::new().get();
        match &self.fastp.metadata {
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
        self.spades = SpadesMetadata::new().get();
        match &self.spades.metadata {
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
        self.lastz = LastzMetadata::new().get();
        match &self.lastz.metadata {
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
        self.mafft = MafftMetadata::new().get();
        match &self.mafft.metadata {
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
        self.iqtree = IqtreeMetadata::new().get();
        match &self.iqtree.metadata {
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
