use std::path::PathBuf;

use crate::core::configs::{raw_reads::DEFAULT_RAW_READ_PREFIX, DEFAULT_CONFIG_DIR};
use clap::{builder, Args};

#[derive(Args)]
pub struct CommonRunnerOptions {
    /// Should the SHA256 checksum be checked
    /// before assembling the files
    #[arg(long, help = "Process samples without checking SHA256 checksum")]
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    #[arg(
        long = "process",
        help = "Process samples if true else check for errors only"
    )]
    pub process: bool,
    /// Optional parameters for the assembly process
    #[arg(
        long,
        require_equals = true,
        help = "Optional parameters for the assembly process"
    )]
    pub override_args: Option<String>,
    /// Check config for errors
    #[arg(
        long,
        help = "Continue processing samples without checking the config file"
    )]
    pub skip_config_check: bool,
}

#[derive(Args)]
pub struct CommonInitArgs {
    /// Output directory for the config file
    #[arg(
        short,
        long,
        default_value = DEFAULT_CONFIG_DIR,
        help = "Select a directory for the config file."
    )]
    pub output: PathBuf,
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
    #[arg(
        short,
        long,
        default_value_t = 3,
        help = "Word length for sample names"
    )]
    pub length: usize,
    /// Specify regex to match raw read file names
    /// Default used internal regex to match fastq and fastq.gz files
    #[arg(
        long,
        require_equals = true,
        help = "Specify file extension to match raw read files. Support regex."
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
    /// Optional prefix for the output files
    #[arg(
        short,
        long,
        default_value = DEFAULT_RAW_READ_PREFIX,
        help = "Prefix for the output files"
    )]
    pub output_prefix: String,
}
