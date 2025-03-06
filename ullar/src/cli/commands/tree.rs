use std::path::PathBuf;

use clap::{builder, Args, Subcommand};

use crate::core::tree::DEFAULT_PHYLO_OUTPUT_DIR;

use super::common::CommonRunnerArgs;


#[derive(Subcommand)]
pub(crate) enum TreeInferenceSubcommand {
    /// Create tree inference config file
    #[command(name = "init", about = "Create tree inference config file")]
    Init(Box<TreeInferenceInitArgs>),
    /// Run tree inference
    #[command(name = "run", about = "Run tree inference")]
    Run(Box<TreeInferenceArgs>),
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
    /// Phylogenetic tree inference method
    #[arg(
        num_args(..=4),
        long,
        help = "Phylogenetic tree inference method",
        value_parser = builder::PossibleValuesParser::new(["ml-species", "ml-gene", "gsc", "msc"])
    )]
    pub specify_methods: Option<Vec<String>>,
    /// Sequence data type. 
    /// Uses by SEGUL (https://segul.app) to parse the alignment files.
    /// We use DNA as the default because the pipeline
    /// is optimized for DNA sequences.
    /// You can use other options that SEGUL supports.
    #[arg(
        short,
        long,
        default_value = "dna",
        help = "Sequence data type. Default is DNA, other options are amino acid",
        value_parser = builder::PossibleValuesParser::new(["ignore","dna", "aa"])
    )]
    pub datatype: String,
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
    #[command(flatten)]
    pub common: CommonRunnerArgs,
}

#[derive(Args)]
pub struct IqTreeSettingArgs {
    /// Path to the partition file
    #[arg(short, long, help = "Path to the partition file")]
    pub partition: Option<PathBuf>,
    /// Partitioning scheme
    #[arg(
        short='P', 
        long="partition-model", 
        help = "Partition model for IQ-TREE",
        default_value = "equal", 
        value_parser = builder::PossibleValuesParser::new(["equal", "proportional", "unlinked"])
    )]
    pub partition_model: String,
    /// Model of nucleotide substitution
    #[arg(
        short,
        long,
        default_value = "GTR+G+I",
        help = "Model of nucleotide substitution for IQ-TREE."
    )]
    pub models: String,
    /// Set different models for gene tree inference
    #[arg(
        short='M',
        long,
        help = "Set different models for gene tree inference"
    )]
    pub gene_models: Option<String>,
    /// Override arguments for IQ-TREE 
    /// species tree inference.
    /// Example: -m GTR+G+I -T 2 -B 1000
    #[arg(
        long,
        help = "Override arguments for IQ-TREE species tree inference"
    )]
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
    pub override_args_genes: Option<String>,
}
