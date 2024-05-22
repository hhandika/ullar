use std::path::PathBuf;

use clap::{
    builder, crate_authors, crate_description, crate_name, crate_version, Args, Parser, Subcommand,
};

use crate::core::new::configs::DEFAULT_RAW_READ_FILENAME;

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
/// Main command line arguments
pub struct UllarCli {
    #[command(subcommand)]
    /// Internal subcommands
    pub(crate) sub_cmd: SubCommand,
    /// Set using interactive mode
    #[arg(long, help = "Set using interactive mode")]
    pub(crate) interactive: bool,
    /// Log directory for the log file
    #[arg(
        long,
        default_value = "logs",
        help = "Select a directory for the log file."
    )]
    /// Prefix for the log file
    pub(crate) log_dir: String,
    #[arg(
        long,
        default_value = "ullar",
        help = "Select a prefix for the log file."
    )]
    pub(crate) log_prefix: String,
}

#[derive(Subcommand)]
pub(crate) enum SubCommand {
    /// New subcommand to init a new project
    #[command(name = "new", about = "Initialize a new project")]
    New(NewArgs),
    /// Clean raw reads
    #[command(name = "clean", about = "Clean raw reads")]
    Clean(CleanArgs),
    /// Subcommand for utility functions
    #[command(subcommand, name = "utils", about = "Utility functions")]
    Utils(UtilSubCommand),
}


#[derive(Subcommand)]
pub(crate) enum UtilSubCommand {
    /// Check and manage dependencies
    #[command(name = "deps", about = "Check and manage dependencies")]
    CheckDepsSubcommand,
    /// Subcommand to hash files
    #[command(name = "checksum", about = "Hash files")]
    ChecksumSubcommand(Sha256Args),
    /// Scan directory for files
    #[command(subcommand, name = "scan", about = "Scan directory for files")]
    ScanSubcommand(ScannerSubcommand),
}

#[derive(Subcommand)]
pub(crate) enum ScannerSubcommand {
    /// Subcommand to scan reads
    #[command(name = "read", about = "Scan reads")]
    ReadSubCommand(ReadScanArgs),
}


#[derive(Args)]
pub struct CleanArgs {
    /// Path to the raw read configuration file
    #[arg(short, long, help = "Path to the raw read configuration file")]
    pub config: PathBuf,
    /// Should the SHA256 checksum be checked
    /// before cleaning the files
    #[arg(long, help = "Process samples without checking SHA256 checksum")]
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    #[arg(long = "process", help = "Process samples if true else check for errors only")]
    pub process_samples: bool,
    /// Output directory to store the cleaned reads
    /// Default used 'cleaned_reads'
    #[arg(
        short,
        long,
        default_value = "cleaned_reads",
        help = "Output directory to store the cleaned reads"
    )]
    pub output: PathBuf,
    /// Arg to allow user to input optional parameters
    #[arg(
        short,
        long,
        help = "Optional parameters for the cleaning process"
    )]
    pub optional_params: Option<String>,
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
    /// Output directory for the config file
    #[arg(
        short,
        long,
        default_value = DEFAULT_RAW_READ_FILENAME,
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
}

#[derive(Args)]
pub struct Sha256Args {
    /// Path to the file to hash
    /// Supports multiple files
    #[arg(short, long, help = "Input file(s) to hash")]
    pub dir: PathBuf,
    /// Match file formats for generic file search
    /// Support fastq, fasta, nexus, phylip, and plain text
    #[arg(
        short, 
        long , 
        help = "Specify input format",
        value_parser = builder::PossibleValuesParser::new([
            "fastq", "fasta", "nexus", "phylip", "text"
        ])
    )]
    pub format: String,
    /// Output file for the hash
    #[arg(short, long, default_value = "sha256", help = "Output file for the hash")]
    pub output: PathBuf,
    /// Use stdout for the output
    #[arg(long, help = "Use stdout for the output")]
    pub stdout: bool,
    /// Find files recursively
    #[arg(long, help = "Find files recursively")]
    pub recursive: bool,
}

#[derive(Args)]
pub struct ReadScanArgs {
    /// Path to the directory to scan
    #[arg(short, long, help = "Input directory to scan")]
    pub dir: PathBuf,
    /// Match file formats for generic file search
    /// Support fastq, fasta, nexus, phylip, and plain text
    #[arg(short, long, default_value = "scan", help = "Specify output path")]
    pub output: PathBuf,
    /// Use stdout for the output
    #[arg(long, help = "Use stdout for the output")]
    pub stdout: bool,
    /// Find files recursively
    #[arg(long, help = "Find files recursively")]
    pub recursive: bool,
    /// Sample name format for matching reads
    /// Default used simple name format
    #[arg(
        long,
        default_value = "simple",
        help = "Sample name format",
        value_parser = builder::PossibleValuesParser::new(["simple", "descriptive"])
    )]
    pub sample_name: String,
}