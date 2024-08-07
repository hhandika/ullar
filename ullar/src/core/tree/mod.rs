use std::{error::Error, fs, path::Path};

use colored::Colorize;

use crate::{
    cli::commands::tree::TreeArgs,
    helper::common,
    types::{Task, TreeInferenceMethod},
};

use super::configs::trees::TreeInferenceConfig;

pub mod iqtree;

pub const DEFAULT_PHYLO_OUTPUT_DIR: &str = "phylogenetic_tree";

pub const DEFAULT_ML_OUTPUT_DIR: &str = "ml_iqtree";

pub const DEFAULT_MSC_OUTPUT_DIR: &str = "msc_aster";

pub struct TreeEstimation<'a> {
    /// Path to raw read configuration file
    pub config_path: &'a Path,
    /// Checksum verification flag
    pub ignore_checksum: bool,
    /// Process samples flag
    pub process_samples: bool,
    /// Parent output directory
    pub output_dir: &'a Path,
    /// Optional parameters
    /// for the phylogenetic estimation
    pub optional_params: Option<&'a str>,
    /// Skip config check flag  
    pub skip_config_check: bool,
    /// Tree inference method
    pub method: TreeInferenceMethod,
    task: Task,
}

impl<'a> TreeEstimation<'a> {
    /// Initialize a new TreeEstimation instance
    /// with the given parameters
    pub fn new(
        config_path: &'a Path,
        ignore_checksum: bool,
        process_samples: bool,
        output_dir: &'a Path,
        optional_params: Option<&'a str>,
        skip_config_check: bool,
        method: TreeInferenceMethod,
    ) -> Self {
        Self {
            config_path,
            ignore_checksum,
            process_samples,
            output_dir,
            optional_params,
            skip_config_check,
            method,
            task: Task::TreeInference,
        }
    }

    /// Initialize a new TreeEstimation instance
    /// from command line arguments
    pub fn from_arg(args: &'a TreeArgs) -> Self {
        Self {
            config_path: &args.config,
            ignore_checksum: args.common.ignore_checksum,
            process_samples: args.common.process,
            output_dir: &args.output,
            optional_params: args.common.override_args.as_deref(),
            skip_config_check: args.common.skip_config_check,
            method: args
                .method
                .parse()
                .expect("Failed to parse tree inference method. Supported methods: all, ml, msc"),
            task: Task::TreeInference,
        }
    }

    pub fn run(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        if self.process_samples {
            self.run_tree_inference();
        }
    }

    fn parse_config(&self) -> Result<TreeInferenceConfig, Box<dyn Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: TreeInferenceConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    fn log_input(&self, config: &TreeInferenceConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        match &config.alignments {
            Some(file) => {
                log::info!("{:18}: {}", "Alignment counts", file.alignments.len());
            }
            None => {
                log::info!("{:18}: {}", "Alignments", "Not provided");
            }
        }
        match &config.trees {
            Some(file) => {
                log::info!("{:18}: {}", "Tree counts", file.trees.len());
            }
            None => {
                log::info!("{:18}: {}", "Trees", "Not provided");
            }
        }
    }

    fn run_tree_inference(&self) {
        match self.method {
            TreeInferenceMethod::All => self.infer_all_trees(),
            TreeInferenceMethod::MLSpeciesTree => self.infer_ml_tree(),
            TreeInferenceMethod::MLGeneTree => self.infer_ml_gene_tree(),
            TreeInferenceMethod::GeneSiteConcordance => self.infer_gsc_tree(),
            TreeInferenceMethod::MSCSpeciesTree => self.infer_msc_tree(),
        }
    }

    fn infer_all_trees(&self) {
        self.infer_ml_tree();
        self.infer_ml_gene_tree();
        self.infer_gsc_tree();
        self.infer_msc_tree();
    }

    fn infer_ml_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating ML species tree");
        spinner.finish_with_message("Finished estimating ML species tree\n");
    }

    fn infer_ml_gene_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating ML gene tree");
        spinner.finish_with_message("Finished estimating ML gene tree\n");
    }

    // Gene Site Concordance Factor
    fn infer_gsc_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating Gene Site Concordance Factor");
        spinner.finish_with_message("Finished estimating Gene Site Concordance Factor\n");
    }

    fn infer_msc_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating MSC species tree");
        spinner.finish_with_message("Finished estimating MSC species tree\n");
    }
}
