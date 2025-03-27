use std::path::PathBuf;

use clap::{
    builder, Args, Subcommand,
};

#[derive(Subcommand)]
pub(crate) enum UtilSubCommand {
    /// Subcommand to hash files
    #[command(name = "checksum", about = "Hash files")]
    Checksum(Sha256Args),
    /// Scan directory for files
    #[command(subcommand, name = "scan", about = "Scan directory for files")]
    Scan(ScannerSubcommand),
    /// Extra function to create symlink on POSIX system
    #[cfg(target_family = "unix")]
    #[command(name = "symlink", about = "Create symlink on POSIX system")]
    Symlink(SymlinkArgs),
    #[command(name = "rename", about = "Rename files or directories")]
    Rename(RenameArgs),
}

#[derive(Subcommand)]
pub(crate) enum ScannerSubcommand {
    /// Subcommand to scan reads
    #[command(name = "read", about = "Scan reads")]
    ReadSubCommand(ReadScanArgs),
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

#[cfg(target_family = "unix")]
#[derive(Args)]
pub struct SymlinkArgs {
    /// Path to the file to link
    #[cfg(target_family = "unix")]
    #[arg(short, long, help = "Input directory to scan")]
    pub dir: PathBuf,
    /// Path to the symlink
    #[cfg(target_family = "unix")]
    #[arg(short, long, default_value = "symlinks", help = "Path to the symlink")]
    pub output: PathBuf,
    /// Supported format
    #[cfg(target_family = "unix")]
    #[arg(
        short, 
        long , 
        default_value = "contigs",
        help = "Specify input format",
        value_parser = builder::PossibleValuesParser::new([
            "contigs", "fastq", "fasta", "nexus", "phylip", "text"
        ])
    )]
    pub format: String,
}

#[derive(Args)]
pub struct RenameArgs {
    /// Path to the directory to rename
    #[arg(short, long, help = "Input directory to rename")]
    pub dir: PathBuf,
    /// Path to the name sources
    #[arg(short, long, help = "Path to the name sources")]
    pub name_sources: PathBuf,
    /// Flag to indicate if the input is a directory
    #[arg(long, default_value_t = false, help = "Flag to indicate if the input is a directory")]
    pub is_dir: bool,
}