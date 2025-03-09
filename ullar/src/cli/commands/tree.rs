use std::path::PathBuf;

use clap::{builder::{self, PossibleValuesParser}, Args, Subcommand};

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
        short = 'f',
        long,
        default_value = "auto",
        help = "Input format of the alignment files",
        value_parser = PossibleValuesParser::new(["auto", "fasta", "phylip", "nexus"])
    )]
    pub input_format: String,
    /// Phylogenetic tree inference method options:
    /// 1. Maximum likelihood species tree inference (ml-species).
    /// 2. Maximum likelihood gene tree inference (ml-gene)
    /// 3. Gene species concordance (gsc)
    /// 4. Multi-species coalescent (msc)
    /// Notes: Species tree inference is a phylogenetic tree inference
    /// using a concatenated alignment. The alignments can be species-level
    /// samples or population-level samples. We use this term to distinguish
    /// it from gene tree inference, which is a phylogenetic estimate infers
    /// for each gene.
    #[arg(
        num_args(..=4),
        long,
        help = "Phylogenetic tree inference method",
        value_parser = PossibleValuesParser::new(["ml-species", "ml-gene", "gscf", "msc"])
    )]
    pub specify_analyses: Option<Vec<String>>,
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
    /// Search recursively for files
    #[arg(long, help = "Search recursively for files")]
    pub recursive: bool,
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
    /// equal: Equal rates for all partitions (-q option in IQ-TREE)
    /// proportional: Proportional rates for all partitions (-spp option in IQ-TREE)
    /// unlinked: Unlinked models for all partitions (-sp option in IQ-TREE)
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
        help = "Model of nucleotide substitution for IQ-TREE. Must matches IQ-TREE model format"
    )]
    pub models: String,
    /// Set different models for gene tree inference
    /// Otherwise, the same model will be used for both
    /// species and gene tree inference.
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
    /// It will be overridden by override_args_species
    #[arg(short, long, default_value = "1", help = "Number of threads to use for IQ-TREE.")]
    pub threads: String,
    /// Number of bootstrap replicates
    #[arg(
        short,
        long,
        help = "Number of bootstrap replicates for IQ-TREE. Default is 1000 for species tree inference"
    )]
    pub bootstrap: Option<String>,
    /// Optional arguments for IQ-TREE
    /// Use the additional arguments other than other 
    /// options provided by other args.
    /// This is different from override_args_species
    /// and override_args_genes, which will override
    /// the arguments provided by the other args.
    #[arg(
        long,
        help = "Optional arguments for IQ-TREE"
    )]
    pub optional_args_species: Option<String>,
    /// Optional arguments for IQ-TREE
    /// gene tree inference. Behaves the same as
    /// optional_args_species.
    #[arg(
        long,
        help = "Optional arguments for IQ-TREE gene tree inference"
    )]
    pub optional_args_genes: Option<String>,
    /// Override arguments for IQ-TREE
    /// species tree inference. It will override
    /// bootstrap, threads, and models. 
    /// DOES NOT include partition arguments.
    /// ULLAR will parse the arguments that match 
    /// models, threads, and bootstrap.
    /// Example: -m GTR+G+I -T 2 -B 1000
    /// Additional arguments will be considered as
    /// optional arguments.
    /// For example, -m GTR+G+I -T 2 -B 1000 -alrt 1000
    /// will set the model to GTR+G+I, threads to 2, and
    /// bootstrap to 1000. The -alrt 1000 will be
    /// considered as optional arguments.
    #[arg(
        long,
        help = "Override arguments for IQ-TREE species tree inference"
    )]
    pub override_args_species: Option<String>,
    /// Optional argument for IQ-TREE gene site concordance factor
    #[arg(
        long,
        help = "Optional argument for IQ-TREE gene site concordance factor"
    )]
    pub optional_args_gscf: Option<String>,
    /// Override arguments for IQ-TREE
    /// gene tree inference.
    #[arg(
        long,
        help = "Override arguments for IQ-TREE gene tree inference"
    )]
    pub override_args_genes: Option<String>,
    /// Recompute likelihoods for gene-site concordance factors.
    /// By default, ULLAR will run IQ-TREE using models from
    /// species tree inference.
    /// The model detection will look for file with extension "best_model.nex".
    /// This method will speed up the process for large datasets.
    /// Learn more here: 
    /// http://www.iqtree.org/doc/Concordance-Factor#gene-concordance-factor-gcf
    /// When this option is set, ULLAR will run IQ-TREE gscf
    /// without model detection.
    #[arg(
        long,
        help = "Recompute likelihoods for gene-site concordance factors"
    )]
    pub recompute_likelihoods: bool,
}
