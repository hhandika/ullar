//! Main command data structure for the CLI
//! Handle all the subcommands, arguments, and flags
pub mod alignment;
pub mod assembly;
pub mod clean;
pub mod common;
pub mod deps;
pub mod map;
pub mod new;
pub mod tree;
pub mod utils;

use alignment::AlignmentSubcommand;
use assembly::AssemblySubcommand;
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser, Subcommand};
use clean::ReadCleaningSubcommand;
use deps::DepsSubcommand;
use map::MapSubcommand;
use new::NewArgs;
use tree::TreeInferenceSubcommand;
use utils::UtilSubCommand;

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
/// Main command line arguments
pub struct UllarCli {
    #[command(subcommand)]
    /// Internal subcommands
    pub(crate) sub_cmd: UllarSubcommand,
    /// Set using interactive mode
    #[arg(long, help = "Set using interactive mode")]
    pub(crate) interactive: bool,
    /// Log directory for the log file
    #[arg(
        long,
        default_value = "logs",
        help = "Select a directory for the log file."
    )]
    /// Prefix for the log file
    pub(crate) log_dir: String,
    #[arg(
        long,
        default_value = "ullar",
        help = "Select a prefix for the log file."
    )]
    pub(crate) log_prefix: String,
}

#[derive(Subcommand)]
pub(crate) enum UllarSubcommand {
    /// New subcommand to init a new project
    #[command(name = "new", about = "Start a new project")]
    New(NewArgs),
    /// Clean raw reads
    #[command(subcommand, name = "clean", about = "Clean raw reads")]
    Clean(ReadCleaningSubcommand),
    /// Assemble cleaned reads
    #[command(subcommand, name = "assemble", about = "Assemble cleaned reads")]
    Assemble(AssemblySubcommand),
    /// Map contigs to reference
    #[command(subcommand, name = "map", about = "Map contigs to reference")]
    Map(MapSubcommand),
    /// Locus alignment
    #[command(subcommand, name = "align", about = "Align sequences")]
    Alignment(AlignmentSubcommand),
    /// Phylogenetic tree estimation
    #[command(subcommand, name = "tree", about = "Estimate phylogenetic tree")]
    Tree(TreeInferenceSubcommand),
    /// For checking dependencies
    #[command(subcommand, name = "deps", about = "Check and manage dependencies")]
    Deps(DepsSubcommand),
    /// Subcommand for utility functions
    #[command(
        subcommand,
        name = "utils",
        about = "Generate SHA256, scan directories, and create symlinks"
    )]
    Utils(UtilSubCommand),
}
