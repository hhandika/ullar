use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::core::clean::DEFAULT_CLEAN_READ_OUTPUT_DIR;

use super::common::{CommonInitArgs, CommonRunnerArgs};

#[derive(Subcommand)]
pub(crate) enum ReadCleaningSubcommand {
    /// Initialize a new clean config file
    #[command(name = "init", about = "Create a clean read config file")]
    Init(ReadCleaningInitArgs),
    /// Clean raw reads
    #[command(name = "run", about = "Clean raw reads")]
    Clean(ReadCleaningArgs),
}

#[derive(Args)]
pub struct ReadCleaningInitArgs {
    /// Input directory containing raw reads
    #[arg(short, long, help = "Input directory containing raw reads")]
    pub dir: PathBuf,
    #[command(flatten)]
    pub common: CommonInitArgs,
}

#[derive(Args)]
pub struct ReadCleaningArgs {
    /// Path to the raw read config file
    #[arg(short, long, help = "Path to the raw read config file")]
    pub config: PathBuf,
    /// Share command across features
    #[command(flatten)]
    pub common: CommonRunnerArgs,
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
