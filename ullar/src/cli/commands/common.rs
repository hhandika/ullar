use clap::{builder, Args};
use std::path::PathBuf;

use crate::helper::configs::DEFAULT_CONFIG_DIR;

#[derive(Args)]
pub struct CommonRunnerArgs {
    /// Should the SHA256 checksum be checked
    /// before assembling the files
    #[arg(long, help = "Process samples without checking SHA256 checksum")]
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    #[arg(
        long = "dry-run",
        help = "Check the config file without processing samples"
    )]
    pub dry_run: bool,
    /// Check config for errors
    #[arg(
        long,
        help = "Continue processing samples without checking the config file"
    )]
    pub skip_config_check: bool,
    /// Force overwrite of existing files
    #[arg(long, help = "Force overwrite of existing files")]
    pub overwrite: bool,
}

#[derive(Args)]
pub struct GenomicReadsInitArgs {
    /// Split separator for sample names
    /// Default used '_'
    /// Example: sample1_R1.fastq.gz -> sample1
    #[arg(short, long, help = "Split separator for sample names")]
    pub separator: Option<char>,
    /// Sample name format
    /// Default used simple name format
    /// where only the first word is captured
    /// Example: sample1_R1.fastq.gz -> sample1
    #[arg(
        long,
        default_value = "descriptive",
        help = "Sample name format",
        value_parser = builder::PossibleValuesParser::new(["simple", "descriptive"])
    )]
    pub sample_name: String,
    /// Word length for sample names
    /// Default used 3: genus_species_museumNumber
    #[arg(short, long, help = "Word length for sample names")]
    pub length: Option<usize>,
    /// Specify regex to match filenames
    /// Default used internal regex to match fastq and fastq.gz files
    #[arg(
        long,
        require_equals = true,
        help = "Specify input file extension to match. Support regex."
    )]
    pub extension: Option<String>,
    /// Specify regex to match sample names
    /// Default used internal regex based on name format.
    #[arg(
        long,
        require_equals = true,
        help = "Specify regex to match sample names"
    )]
    pub re_sample: Option<String>,
    /// Search recursively for files
    #[arg(long, help = "Search recursively for files")]
    pub recursive: bool,
}

#[derive(Args)]
pub struct CommonInitArgs {
    /// Output directory to store the config file
    #[arg(
        short,
        long,
        default_value = DEFAULT_CONFIG_DIR,
        help = "Output directory to write the config file"
    )]
    pub output: PathBuf,
    /// Optional parameters for runner
    #[arg(
        long,
        require_equals = true,
        help = "Optional parameters for the runner"
    )]
    pub override_args: Option<String>,
}
