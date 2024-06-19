use std::path::PathBuf;

use clap::Args;

use crate::core::assembly::DEFAULT_ASSEMBLY_OUTPUT_DIR;

use super::common::CommonRunnerOptions;

#[derive(Args)]
pub struct AssemblyArgs {
    /// Path to the assembly configuration file
    #[arg(short, long, help = "Path to the assembly configuration file")]
    pub config: PathBuf,
    /// Output directory to store the assemblies
    #[arg(short, long, default_value = DEFAULT_ASSEMBLY_OUTPUT_DIR,
        help = "Output directory to store the assemblies")]
    pub output: PathBuf,
    #[command(flatten)]
    pub common: CommonRunnerOptions,
    /// Remove SPAdes intermediate files
    #[arg(long, help = "Remove SPAdes intermediate files")]
    pub keep_intermediates: bool,
    /// Rename contigs file to sample name
    #[arg(long, help = "Rename contigs file to sample name")]
    pub rename_contigs: bool,
}
