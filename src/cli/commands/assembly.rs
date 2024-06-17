use std::path::PathBuf;

use clap::Args;

use crate::core::assembly::DEFAULT_ASSEMBLY_OUTPUT_DIR;

#[derive(Args)]
pub struct AssemblyArgs {
    /// Path to the assembly configuration file
    #[arg(short, long, help = "Path to the assembly configuration file")]
    pub config: PathBuf,
    /// Should the SHA256 checksum be checked
    /// before assembling the files
    #[arg(long, help = "Process samples without checking SHA256 checksum")]
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    #[arg(
        long = "process",
        help = "Process samples if true else check for errors only"
    )]
    pub process_samples: bool,
    /// Output directory to store the assemblies
    #[arg(short, long, default_value = DEFAULT_ASSEMBLY_OUTPUT_DIR,
        help = "Output directory to store the assemblies")]
    pub output: PathBuf,
    /// Optional parameters for the assembly process
    #[arg(short, long, help = "Optional parameters for the assembly process")]
    pub optional_params: Option<String>,
    /// Check config for errors
    #[arg(
        long,
        help = "Continue processing samples without checking the config file"
    )]
    pub skip_config_check: bool,
    /// Remove SPAdes intermediate files
    #[arg(long, help = "Remove SPAdes intermediate files")]
    pub keep_intermediates: bool,
    /// Rename contigs file to sample name
    #[arg(long, help = "Rename contigs file to sample name")]
    pub rename_contigs: bool,
}
