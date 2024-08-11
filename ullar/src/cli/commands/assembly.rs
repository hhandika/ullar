use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::assembly::DEFAULT_ASSEMBLY_OUTPUT_DIR;

use super::common::{CommonInitArgs, CommonRunnerOptions};

#[derive(Subcommand)]
pub(crate) enum AssemblySubcommand {
    /// Create assembly configuration file
    #[command(name = "init", about = "Create assembly configuration file")]
    Init(AssemblyInitArgs),
    /// Assemble cleaned reads
    #[command(name = "run", about = "Assemble cleaned reads")]
    Assembly(AssemblyArgs),
}

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

#[derive(Args)]
pub struct AssemblyInitArgs {
    /// Input directory containing the cleaned reads
    #[arg(short, long, help = "Input directory containing the cleaned reads")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}
