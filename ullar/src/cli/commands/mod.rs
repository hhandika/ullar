pub mod assembly;
pub mod clean;
mod common;
pub mod deps;
pub mod init;
pub mod tree;
pub mod utils;

use assembly::AssemblyArgs;
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser, Subcommand};
use clean::{CleanArgs, CleanSubcommand};
use deps::DepsSubcommand;
use init::{InitSubCommand, NewArgs};
use tree::TreeArgs;
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
    /// Initialize config file to allow starting from any step
    /// of the pipeline workflow.
    #[command(
        subcommand,
        name = "init",
        about = "Initialize config files. Start from any step of the workflow."
    )]
    Init(InitSubCommand),
    /// Clean raw reads
    #[command(subcommand, name = "clean", about = "Clean raw reads")]
    Clean(CleanSubcommand),
    /// Assemble cleaned reads
    #[command(name = "assemble", about = "Assemble cleaned reads")]
    Assemble(AssemblyArgs),
    /// Map contigs to reference
    #[command(name = "map", about = "Map contigs to reference")]
    Map,
    /// Phylogenetic tree estimation
    #[command(name = "tree", about = "Phylogenetic tree estimation")]
    Tree(TreeArgs),
    /// For checking dependencies
    #[command(subcommand, name = "deps", about = "Check and manage dependencies")]
    Deps(DepsSubcommand),
    /// Subcommand for utility functions
    #[command(subcommand, name = "utils", about = "Utility functions")]
    Utils(UtilSubCommand),
}
