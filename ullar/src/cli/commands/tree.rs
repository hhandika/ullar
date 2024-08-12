use std::path::PathBuf;

use clap::Args;

use crate::core::tree::DEFAULT_PHYLO_OUTPUT_DIR;

use super::common::CommonRunnerArgs;

#[derive(Args)]
pub struct TreeArgs {
    /// Path to the phylogenetic estimation configuration file
    #[arg(
        short,
        long,
        help = "Path to the phylogenetic estimation configuration file"
    )]
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
