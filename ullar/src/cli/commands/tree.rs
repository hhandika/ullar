use std::path::PathBuf;

use clap::{builder, Args, Subcommand};

use crate::core::tree::DEFAULT_PHYLO_OUTPUT_DIR;


#[derive(Subcommand)]
pub(crate) enum TreeInferenceSubcommand {
    /// Create tree inference config file
    #[command(name = "init", about = "Create tree inference config file")]
    Init(TreeInferenceInitArgs),
    /// Run tree inference
    #[command(name = "run", about = "Run tree inference")]
    Run(TreeInferenceArgs),
}

#[derive(Args)]
pub struct TreeInferenceInitArgs {
    /// Input directory containing the alignment files
    #[arg(
        short,
        long,
        default_value = DEFAULT_PHYLO_OUTPUT_DIR,
        help = "Input directory containing the alignment files"
    )]
    pub dir: PathBuf,
    /// Input format of the alignment files
    #[arg(
        short,
        long,
        default_value = "auto",
        help = "Input format of the alignment files"
    )]
    pub input_format: String,
    /// Path to the partition file
    #[arg(short, long, help = "Path to the partition file")]
    pub partition: Option<PathBuf>,
    /// Phylogenetic tree inference method
    #[arg(short, long, help = "Phylogenetic tree inference method")]
    pub method: Option<String>,
    #[command(flatten)]
    pub iqtree: IqTreeSettingArgs,
    
}

#[derive(Args)]
pub struct TreeInferenceArgs {
    /// Path to the phylogenetic estimation config file
    #[arg(short, long, help = "Path to the phylogenetic estimation config file")]
    pub config: Option<PathBuf>,
    /// Output directory to store the phylogenetic trees
    #[arg(short, long, default_value = DEFAULT_PHYLO_OUTPUT_DIR,
        help = "Output directory to store the phylogenetic trees")]
    pub output: PathBuf,
    /// Phylogenetic tree inference method
    #[arg(
        short,
        long,
        default_value = "all",
        help = "Phylogenetic tree inference method",
        value_parser = builder::PossibleValuesParser::new(["all", "ml-species", "ml-gene", "gsc", "msc"])
    )]
    pub method: String,
}

#[derive(Args)]
pub struct IqTreeSettingArgs {
    /// Model of nucleotide substitution
    #[arg(
        short,
        long,
        default_value = "GTR+G+I",
        help = "Model of nucleotide substitution"
    )]
    pub models: String,
    /// Number of threads to use
    #[arg(short, long, default_value = "1", help = "Number of threads to use for IQ-TREE")]
    pub threads: String,
    /// Number of bootstrap replicates
    #[arg(
        short,
        long,
        default_value = "1000",
        help = "Number of bootstrap replicates for IQ-TREE"
    )]
    pub bootstrap: String,
    /// Partitioning scheme
    #[arg(
        short, 
        long, 
        help = "Partition model for IQ-TREE",
        default_value = "equal", 
        value_parser = builder::PossibleValuesParser::new(["equal", "proporsional", "unlinked"])
    )]
    pub partition: String,
    /// Override arguments for IQ-TREE 
    /// species tree inference.
    /// Example: -m GTR+G+I -T 2 -B 1000
    #[arg(
        long,
        help = "Override arguments for IQ-TREE species tree inference"
    )]
    pub override_args_species: Option<String>,
    /// Override arguments for IQ-TREE
    /// gene tree inference.
    #[arg(
        long,
        help = "Override arguments for IQ-TREE gene tree inference"
    )]
    pub override_args_gene: Option<String>,
}
