use clap::{crate_authors, crate_description, crate_name, crate_version, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
/// Main command line arguments
pub(crate) struct UllarCli {
    #[command(subcommand)]
    /// Internal subcommands
    pub(crate) sub_cmd: SubCommand,
}

#[derive(Subcommand)]
pub(crate) enum SubCommand {
    /// New subcommand to init a new project
    #[command(name = "init", about = "Initialize a new project")]
    Init(NewArgs),
}

#[derive(Args)]
pub(crate) struct NewArgs {
    /// Name of the project
    #[arg(
        short,
        long,
        default_value = "raw_reads",
        help = "Select a directory for the raw read location."
    )]
    pub(crate) dir: String,
    /// Output directory for the config file
    #[arg(
        short,
        long,
        default_value = "configs",
        help = "Select a directory for the config file."
    )]
    pub(crate) output: String,
    /// Split separator for sample names
    /// Default used '_'
    /// Example: sample1_R1.fastq.gz -> sample1
    #[arg(short, long, help = "Split separator for sample names")]
    pub(crate) separator: Option<String>,
    /// Word length for sample names
    /// Default used 3: genus_species_museumNumber
    #[arg(
        short,
        long,
        default_value_t = 3,
        help = "Word length for sample names"
    )]
    pub(crate) length: usize,
    /// Specify regex to match raw read file names
    /// Default used internal regex to match fastq and fastq.gz files
    #[arg(
        long,
        require_equals = true,
        help = "Specify regex to match raw read file names"
    )]
    pub(crate) re_file: Option<String>,
    /// Specify regex to match sample names
    /// Default used split by _ and - to match sample names
    #[arg(
        long,
        require_equals = true,
        help = "Specify regex to match sample names"
    )]
    pub(crate) re_sample: Option<String>,
}
