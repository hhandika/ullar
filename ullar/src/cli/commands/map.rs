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
    Contig(MapContigArgs),
    /// Perform read mapping
    #[command(name = "read", about = "Map reads to reference sequences")]
    Read(MapReadArgs),
}

#[derive(Args)]
pub struct MapContigArgs {
    /// Path to the map configuration file
    #[arg(short, long, help = "Path to the map configuration file")]
    pub config: PathBuf,
    /// Path to the reference sequence
    #[arg(short, long, help = "Path to the reference sequence")]
    pub reference: PathBuf,
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
    /// Input directory containing query sequences
    #[arg(short, long, required_unless_present = "input", help = "Path to the directory containing query sequences")]
    pub dir: Option<PathBuf>,
    /// Input query path.
    #[arg(
        short,
        long,
        default_value = "fasta",
        conflicts_with = "query-dir",
        num_args(0..),
        help = "Input query path using stdin.",
    )]
    pub input: Option<Vec<PathBuf>>,

    #[arg(
        short = 'f',
        long = "format",
        default_value = "contig",
        help = "Input query format.",
        value_parser = builder::PossibleValuesParser::new(["contig", "read"])
    )]
    pub query_format: String,
    #[arg(
        long,
        default_value = "file",
        help = "Sample name sources",
        value_parser = builder::PossibleValuesParser::new(["file", "directory"])
    )]
    pub name_source: String,
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