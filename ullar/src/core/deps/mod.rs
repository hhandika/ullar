//! Utilities for managing dependencies.
use core::panic;

use colored::Colorize;
use comfy_table::{Cell, Color, Table};
use fastp::FastpMetadata;
use iqtree::IqtreeMetadata;
use lastz::LastzMetadata;
use mafft::MafftMetadata;
use segul::get_segul_metadata;
use serde::{Deserialize, Serialize};
use spades::SpadesMetadata;

pub mod fastp;
pub mod iqtree;
pub mod lastz;
pub mod mafft;
pub mod segul;
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

pub fn check_dependency_match(dep: &DepMetadata, config_version: &str) {
    if dep.version != config_version {
        log::warn!(
            "\n{} Installed {} version {} is different from the config version {}.\
            ULLAR will use the installed version",
            "Warning:".yellow(),
            dep.app_name,
            dep.version,
            config_version
        );
    }
}

pub fn dependency_not_found(dep: &str) {
    log::error!(
        "{} {} is not found. 
        Please ensure {} is installed and accessible in your PATH",
        "Error:".red(),
        dep,
        dep
    );
    panic!("{} Dependency not found", "Error:".red());
}

/// Data structure to store dependency metadata
/// Shared by all dependencies
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DepMetadata {
    pub app_name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,
    /// Additional arguments/flags for the executable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    /// Method used if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
}

impl DepMetadata {
    pub fn new(app_name: &str, version: &str, executable: Option<&str>) -> Self {
        Self {
            app_name: app_name.to_string(),
            version: version.to_string(),
            executable: executable.map(|s| s.to_string()),
            override_args: None,
            methods: None,
        }
    }

    pub fn with_methods(mut self, methods: Vec<String>) -> Self {
        self.methods = Some(methods);
        self
    }

    /// We use this to override the default arguments
    /// when mutable reference is possible.
    pub fn set_methods(&mut self, methods: Vec<String>) {
        self.methods = Some(methods);
    }

    pub fn get_executable(&self, default: &str) -> String {
        self.executable
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DependencyCheck {
    fastp: Option<DepMetadata>,
    spades: Option<DepMetadata>,
    lastz: Option<DepMetadata>,
    mafft: Option<DepMetadata>,
    iqtree: Option<DepMetadata>,
    segul: Option<DepMetadata>,
}

impl DependencyCheck {
    pub fn new() -> Self {
        Self {
            fastp: FastpMetadata::new().get(),
            spades: SpadesMetadata::new().get(),
            lastz: LastzMetadata::new().get(),
            mafft: MafftMetadata::new().get(),
            iqtree: IqtreeMetadata::new().get(),
            segul: Some(get_segul_metadata()),
        }
    }

    pub fn with_override_args(override_args: Option<&str>) -> Self {
        Self {
            fastp: FastpMetadata::new().override_args(override_args).get(),
            spades: SpadesMetadata::new().override_args(override_args).get(),
            lastz: LastzMetadata::new().override_args(override_args).get(),
            mafft: MafftMetadata::new().override_args(override_args).get(),
            iqtree: IqtreeMetadata::new().get(),
            segul: Some(get_segul_metadata()),
        }
    }

    pub fn check(&mut self) {
        let mut table = self.log_status();
        self.log_read_cleaning(&mut table);
        self.log_denovo_assembly(&mut table);
        self.log_contig_mapping(&mut table);
        self.log_sequence_alignment(&mut table);
        self.log_phylogenetic_inference(&mut table);
        self.log_data_wrangling_summarization(&mut table);
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
                let cells = self.get_cell(feature, "fastp", &metadata.version, Some(true));
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "fastp", "Unknown", Some(false));
                table.add_row(cells);
            }
        }
    }

    fn log_denovo_assembly(&mut self, table: &mut Table) {
        let feature = "De novo assembly";
        match &self.spades {
            Some(metadata) => {
                let cells = self.get_cell(feature, "SPAdes", &metadata.version, Some(true));
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "SPAdes", "Unknown", Some(false));
                table.add_row(cells);
            }
        }
    }

    fn log_contig_mapping(&mut self, table: &mut Table) {
        let feature = "Contig mapping";
        match &self.lastz {
            Some(metadata) => {
                let cells = self.get_cell(feature, "LASTZ", &metadata.version, Some(true));
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "LASTZ", "Unknown", Some(false));
                table.add_row(cells);
            }
        }
    }

    fn log_sequence_alignment(&mut self, table: &mut Table) {
        let feature = "Sequence alignment";
        match &self.mafft {
            Some(metadata) => {
                let cells = self.get_cell(feature, "MAFFT", &metadata.version, Some(true));
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "MAFFT", "Unknown", Some(false));
                table.add_row(cells);
            }
        }
    }

    fn log_phylogenetic_inference(&mut self, table: &mut Table) {
        let feature = "Phylogenetic inference";
        match &self.iqtree {
            Some(metadata) => {
                let cells = self.get_cell(feature, "IQ-TREE", &metadata.version, Some(true));
                table.add_row(cells);
            }
            None => {
                let cells = self.get_cell(feature, "IQ-TREE", "Unknown", Some(false));
                table.add_row(cells);
            }
        }
    }

    fn log_data_wrangling_summarization(&mut self, table: &mut Table) {
        let feature = "Data shaping, cleaning, and summarization";
        let status = None;
        match &self.segul {
            Some(metadata) => {
                let cells = self.get_cell(feature, "SEGUL", &metadata.version, status);
                table.add_row(cells);
            }
            None => unreachable!("SEGUL should always be available"),
        }
    }

    fn get_cell(&self, feature: &str, app: &str, version: &str, ok: Option<bool>) -> Vec<Cell> {
        let status = self.status_ok(ok);
        vec![
            Cell::new(feature),
            Cell::new(app),
            Cell::new(version),
            status,
        ]
    }

    fn status_ok(&self, ok: Option<bool>) -> Cell {
        match ok {
            None => Cell::new("🔧 BUILT-IN").fg(Color::Blue),
            Some(true) => Cell::new("✅ OK").fg(Color::Green),
            Some(false) => Cell::new("❌ NOT FOUND").fg(Color::Red),
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
