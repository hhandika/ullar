use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::tree::DEFAULT_PHYLO_OUTPUT_DIR;

use super::common::{CommonInitArgs, CommonRunnerArgs};

#[derive(Subcommand)]
pub(crate) enum TreeInferenceSubcommand {
    /// Create tree inference config file
    #[command(name = "init", about = "Create tree inference config file")]
    Init(TreeInferenceInitArgs),
    /// Run tree inference
    #[command(name = "run", about = "Run tree inference")]
    Run(TreeInferenceArgs),
}

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
    /// Path to the partition file
    #[arg(short, long, help = "Path to the partition file")]
    pub partition: Option<PathBuf>,
    /// Phylogenetic tree inference method
    #[arg(short, long, help = "Phylogenetic tree inference method")]
    pub method: Option<String>,
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
