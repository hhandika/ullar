use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::alignment::DEFAULT_ALIGNMENT_OUTPUT_DIR;

use super::common::{CommonInitArgs, CommonRunnerOptions};

#[derive(Subcommand)]
pub(crate) enum AlignmentSubcommand {
    /// Create alignment configuration file
    #[command(name = "init", about = "Create alignment configuration file")]
    Init(AlignmentInitArgs),
    /// Perform locus alignment
    #[command(name = "align", about = "Perform locus alignment")]
    Align(AlignmentArgs),
}

#[derive(Args)]
pub struct AlignmentArgs {
    /// Path to the alignment configuration file
    #[arg(short, long, help = "Path to the alignment configuration file")]
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
    pub common: CommonRunnerOptions,
}

#[derive(Args)]
pub struct AlignmentInitArgs {
    /// Input directory containing the assemblies
    #[arg(short, long, help = "Input directory containing the assemblies")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}
