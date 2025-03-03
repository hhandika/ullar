use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::alignment::DEFAULT_ALIGNMENT_OUTPUT_DIR;

use super::common::{CommonInitArgs, CommonRunnerArgs};

#[derive(Subcommand)]
pub(crate) enum AlignmentSubcommand {
    /// Create alignment config file
    #[command(name = "init", about = "Create an sequence aligning config file")]
    Init(AlignmentInitArgs),
    /// Perform locus alignment
    #[command(name = "align", about = "Align multiple sequences")]
    Align(AlignmentArgs),
}

#[derive(Args)]
pub struct AlignmentArgs {
    /// Path to the alignment config file
    #[arg(short, long, help = "Path to the alignment config file")]
    pub config: PathBuf,
    /// Output directory to store the alignments
    #[arg(
        short,
        long,
        default_value = DEFAULT_ALIGNMENT_OUTPUT_DIR,
        help = "Output directory to store the alignments"
    )]
    pub output: PathBuf,
    #[command(flatten)]
    pub common: CommonRunnerArgs,
}

#[derive(Args)]
pub struct AlignmentInitArgs {
    /// Input directory containing the assemblies
    #[arg(short, long, help = "Input directory containing the assemblies")]
    pub dir: PathBuf,
    #[arg(short, long, help = "Input format of the sequences")]
    pub input_fmt: Option<String>,
    #[command(flatten)]
    pub common: CommonInitArgs,
}
