use std::path::PathBuf;

use clap::{
    builder, crate_authors, crate_description, crate_name, crate_version, Args, Parser, Subcommand,
};

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
/// Main command line arguments
pub(crate) struct UllarCli {
    #[command(subcommand)]
    /// Internal subcommands
    pub(crate) sub_cmd: SubCommand,
    #[arg(
        long,
        default_value = "logs",
        help = "Select a directory for the log file."
    )]
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
    /// Subcommand for utility functions
    #[command(subcommand, name = "utils", about = "Utility functions")]
    Utils(UtilSubCommand),
}

#[derive(Subcommand)]
pub(crate) enum UtilSubCommand {
    /// Subcommand to hash files
    #[command(name = "sha256", about = "Hash files")]
    Sha256SubCommand(Sha256Args),
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
        default_value = "configs",
        help = "Select a directory for the config file."
    )]
    pub output: PathBuf,
    /// Split separator for sample names
    /// Default used '_'
    /// Example: sample1_R1.fastq.gz -> sample1
    #[arg(short, long, help = "Split separator for sample names")]
    pub separator: Option<String>,
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
        help = "Specify regex to match raw read file names"
    )]
    pub re_file: Option<String>,
    /// Specify regex to match sample names
    /// Default used split by _ and - to match sample names
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
