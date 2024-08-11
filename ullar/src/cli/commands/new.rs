use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::configs::DEFAULT_CONFIG_DIR;

use super::common::CommonInitArgs;

#[derive(Subcommand)]
pub(crate) enum InitSubCommand {
    /// Initialize config file for assembly
    #[command(name = "assembly", about = "Initialize config file for assembly")]
    Assembly(AssemblyInitArgs),
    /// Initialize config file for mapping contigs
    #[command(name = "map", about = "Initialize config file for mapping contigs")]
    Map(MapInitArgs),
}

#[derive(Args)]
pub struct NewArgs {
    /// Name of the project
    #[arg(
        short,
        long,
        default_value = "raw_reads",
        help = "Select a directory for the raw read location."
    )]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}

#[derive(Args)]
pub struct InitArgs {
    /// Name of the project
    #[arg(short, long, help = "Select a directory for the raw read location.")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}

#[derive(Args)]
pub struct AssemblyInitArgs {
    /// Path to the assembly input directory
    #[arg(short, long, help = "Path to the assembly input directory")]
    pub dir: PathBuf,
    /// Output directory to store the assemblies
    #[arg(short, long, default_value = DEFAULT_CONFIG_DIR, help = "Output directory to write the config file")]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct MapInitArgs {
    /// Path to the assembly input directory
    #[arg(short, long, help = "Path to the assembly input directory")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
    /// Create symlink for phyluce compatibility
    #[cfg(target_family = "unix")]
    #[arg(long, help = "Create symlink for phyluce compatibility")]
    pub phyluce: bool,
}
