use std::path::PathBuf;

use clap::{builder, Args, Subcommand};

use crate::{core::map::{configs::DEFAULT_REF_MAPPING_CONFIG, DEFAULT_MAPPED_CONTIG_OUTPUT_DIR}, helper::regex::CONTIG_SAMPLE_REGEX};

use super::common::CommonRunnerArgs;

#[derive(Subcommand)]
pub(crate) enum MapSubcommand {
    /// Create a new map config file
    #[command(name = "init", about = "Create a new map config file")]
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
    /// Path to the map config file
    #[arg(short, long, help = "Path to the map config file")]
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
    /// Path to the reference sequence
    #[arg(short, long, help = "Path to the reference sequence")]
    pub reference: PathBuf,
    #[arg(
        long,
        default_value = "^(uce|locus)-\\d+",
        help = "Regular expression to extract reference name",
        require_equals = true,
    )]
    pub re_reference: String,
    #[arg(
        long,
        default_value = CONTIG_SAMPLE_REGEX,
        help = "Regular expression to match sample names",
        require_equals = true,
    )]
    pub re_sample: String,
    #[arg(
        long,
        default_value = "file",
        help = "Sample name sources",
        value_parser = builder::PossibleValuesParser::new(["file", "directory", "regex"])
    )]
    pub name_source: String,
    /// Config file name
    #[arg(
        long,
        default_value = DEFAULT_REF_MAPPING_CONFIG,
        help = "Config file name"
    )]
    pub config_name: String,
    #[command(flatten)]
    pub common: CommonRunnerArgs,
}

#[derive(Args)]
pub struct MapReadArgs {
    /// Path to the map config file
    #[arg(short, long, help = "Path to the map config file")]
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