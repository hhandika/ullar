use std::path::PathBuf;

use clap::Args;

use crate::core::tree::DEFAULT_PHYLO_OUTPUT_DIR;

use super::common::{CommonInitArgs, CommonRunnerArgs};

#[derive(Args)]
pub struct TreeInferenceInitArgs {
    /// Input directory containing the alignment files
    #[arg(
        short,
        long,
        default_value = DEFAULT_PHYLO_OUTPUT_DIR,
        help = "Input directory containing the alignment files"
    )]
    pub dir: PathBuf,
    /// Input format of the alignment files
    #[arg(
        short,
        long,
        default_value = "auto",
        help = "Input format of the alignment files"
    )]
    pub input_format: String,
    #[command(flatten)]
    pub common: CommonInitArgs,
}

#[derive(Args)]
pub struct TreeInferenceArgs {
    /// Path to the phylogenetic estimation config file
    #[arg(short, long, help = "Path to the phylogenetic estimation config file")]
    pub config: PathBuf,
    /// Output directory to store the phylogenetic trees
    #[arg(short, long, default_value = DEFAULT_PHYLO_OUTPUT_DIR,
        help = "Output directory to store the phylogenetic trees")]
    pub output: PathBuf,
    #[command(flatten)]
    pub common: CommonRunnerArgs,
    /// Phylogenetic tree inference method
    #[arg(
        short,
        long,
        default_value = "all",
        help = "Phylogenetic tree inference method"
    )]
    pub method: String,
}
