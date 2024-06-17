use std::path::PathBuf;

use clap::Args;

use crate::core::qc::DEFAULT_CLEAN_READ_OUTPUT_DIR;

#[derive(Args)]
pub struct CleanArgs {
    /// Path to the raw read configuration file
    #[arg(short, long, help = "Path to the raw read configuration file")]
    pub config: PathBuf,
    /// Should the SHA256 checksum be checked
    /// before cleaning the files
    #[arg(long, help = "Process samples without checking SHA256 checksum")]
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    #[arg(
        long = "process",
        help = "Process samples if true else check for errors only"
    )]
    pub process_samples: bool,
    /// Output directory to store the cleaned reads
    /// Default used 'cleaned_reads'
    #[arg(
        short,
        long,
        default_value = DEFAULT_CLEAN_READ_OUTPUT_DIR,
        help = "Output directory to store the cleaned reads"
    )]
    pub output: PathBuf,
    /// Arg to allow user to input optional parameters
    #[arg(short, long, help = "Optional parameters for the cleaning process")]
    pub optional_params: Option<String>,
    /// Check config for errors
    #[arg(
        long,
        help = "Continue processing samples without checking the config file"
    )]
    pub skip_config_check: bool,
}
