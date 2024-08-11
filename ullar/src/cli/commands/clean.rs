use std::path::PathBuf;

use clap::Args;

use crate::core::qc::DEFAULT_CLEAN_READ_OUTPUT_DIR;

use super::common::CommonRunnerOptions;

#[derive(Subcommand)]
pub(crate) enum CleanSubcommand {
    /// Clean raw reads
    #[command(name = "run", about = "Perform read cleaning")]
    Clean(CleanArgs),
}

#[derive(Args)]
pub struct CleanArgs {
    /// Path to the raw read configuration file
    #[arg(short, long, help = "Path to the raw read configuration file")]
    pub config: PathBuf,
    /// Share command across features
    #[command(flatten)]
    pub common: CommonRunnerOptions,
    /// Output directory to store the cleaned reads
    /// Default used 'cleaned_reads'
    #[arg(
        short,
        long,
        default_value = DEFAULT_CLEAN_READ_OUTPUT_DIR,
        help = "Output directory to store the cleaned reads"
    )]
    pub output: PathBuf,
}
