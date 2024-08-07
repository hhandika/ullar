use std::path::PathBuf;

use clap::{builder, Args, Subcommand};

use crate::core::configs::{raw_reads::DEFAULT_RAW_READ_PREFIX, DEFAULT_CONFIG_DIR};

#[derive(Subcommand)]
pub(crate) enum InitSubCommand {
    /// Initialize config file for assembly
    #[command(name = "assembly", about = "Initialize config file for assembly")]
    Assembly(AssemblyInitArgs),
    /// Initialize config file for mapping contigs
    #[command(name = "map", about = "Initialize config file for mapping contigs")]
    Map(MapInitArgs),
}

#[derive(Args)]
pub struct NewArgs {
    /// Name of the project
    #[arg(
        short,
        long,
        default_value = "raw_reads",
        help = "Select a directory for the raw read location."
    )]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}

#[derive(Args)]
pub struct InitArgs {
    /// Name of the project
    #[arg(short, long, help = "Select a directory for the raw read location.")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}

#[derive(Args)]
pub struct AssemblyInitArgs {
    /// Path to the assembly input directory
    #[arg(short, long, help = "Path to the assembly input directory")]
    pub dir: PathBuf,
    /// Output directory to store the assemblies
    #[arg(short, long, default_value = DEFAULT_CONFIG_DIR, help = "Output directory to write the config file")]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct MapInitArgs {
    /// Path to the assembly input directory
    #[arg(short, long, help = "Path to the assembly input directory")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
    /// Create symlink for phyluce compatibility
    #[cfg(target_family = "unix")]
    #[arg(long, help = "Create symlink for phyluce compatibility")]
    pub phyluce: bool,
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
        default_value = "simple",
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
