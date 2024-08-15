use std::path::PathBuf;

use clap::{builder, Args, Subcommand};

use crate::core::map::DEFAULT_MAPPED_CONTIG_OUTPUT_DIR;

use super::common::CommonRunnerArgs;

#[derive(Subcommand)]
pub(crate) enum MapSubcommand {
    /// Create a new map configuration file
    #[command(name = "init", about = "Create a new map configuration file")]
    Init(MapInitArgs),
    /// Perform contig mapping
    #[command(name = "contig", about = "Map contigs to reference sequences")]
    MapContig(MapContigArgs),
    /// Perform read mapping
    #[command(name = "read", about = "Map reads to reference sequences")]
    MapRead(MapReadArgs),
}

#[derive(Args)]
pub struct MapContigArgs {
    /// Path to the map configuration file
    #[arg(short, long, help = "Path to the map configuration file")]
    pub config: PathBuf,
    /// Output directory to store the alignments
    #[arg( 
        short,
        long,
        default_value = DEFAULT_MAPPED_CONTIG_OUTPUT_DIR,
        help = "Output directory to store the alignments"
    )]
    pub output: PathBuf,
    #[command(flatten)]
    pub common: CommonRunnerArgs,
}

#[derive(Args)]
pub struct MapInitArgs {
    /// Input directory containing the target reference sequences
    #[arg(short = 't', long = "target", help = "Input directory containing the target reference sequences")]
    pub target_dir: PathBuf,
    /// Input directory containing query sequences
    #[arg(short = 'q', long = "query", help = "Input directory containing query sequences")]
    pub query_dir: PathBuf,
    /// Input format. Possible values: Fasta, Fastq
    #[arg(
        short = 'Q',
        long = "query-format",
        default_value = "fasta",
        help = "Input query format.",
        value_parser = builder::PossibleValuesParser::new(["fasta", "fastq"])
    )]
    pub query_format: String,
    /// Target format. Possible values: Fasta, Fastq
    #[arg(
        short = 'T',
        long = "target-format",
        default_value = "fasta",
        help = "Input target format.",
        value_parser = builder::PossibleValuesParser::new(["fasta", "fastq"])
    )]
    pub target_format: String,
    #[command(flatten)]
    pub common: CommonRunnerArgs,
}

#[derive(Args)]
pub struct MapReadArgs {
    /// Path to the map configuration file
    #[arg(short, long, help = "Path to the map configuration file")]
    pub config: PathBuf,
    /// Output directory to store the alignments
    #[arg( 
        short,
        long,
        default_value = DEFAULT_MAPPED_CONTIG_OUTPUT_DIR,
        help = "Output directory to store the alignments"
    )]
    pub output: PathBuf,
    #[command(flatten)]
    pub common: CommonRunnerArgs,
}