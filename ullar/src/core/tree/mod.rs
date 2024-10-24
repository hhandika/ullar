use std::{error::Error, fs, path::Path};

use colored::Colorize;
use configs::TreeInferenceConfig;

use crate::{
    cli::commands::tree::TreeInferenceArgs,
    helper::common,
    types::{runner::RunnerOptions, Task, TreeInferenceMethod},
};

pub mod configs;
pub mod init;
pub mod iqtree;

pub const DEFAULT_PHYLO_OUTPUT_DIR: &str = "phylogenetic_tree";
pub const DEFAULT_ML_OUTPUT_DIR: &str = "ml_iqtree";
pub const DEFAULT_MSC_OUTPUT_DIR: &str = "msc_aster";

pub struct TreeEstimation<'a> {
    /// Path to raw read config file
    pub config_path: &'a Path,
    /// Checksum verification flag
    pub ignore_checksum: bool,
    /// Parent output directory
    pub output_dir: &'a Path,
    /// Runner options
    pub runner: RunnerOptions<'a>,
    #[allow(dead_code)]
    task: Task,
}

impl<'a> TreeEstimation<'a> {
    /// Initialize a new TreeEstimation instance
    /// with the given parameters
    pub fn new(config_path: &'a Path, ignore_checksum: bool, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            ignore_checksum,
            output_dir,
            runner: RunnerOptions::default(),
            task: Task::TreeInference,
        }
    }

    /// Initialize a new TreeEstimation instance
    /// from command line arguments
    pub fn from_arg(args: &'a TreeInferenceArgs) -> Self {
        Self {
            config_path: &args.config,
            ignore_checksum: args.common.ignore_checksum,
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::TreeInference,
        }
    }

    pub fn run(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        if config.method.is_empty() {
            log::warn!(
                "{} No tree inference method specified in the config files. Using all methods",
                "Warning:".yellow()
            );
        }
        self.run_tree_inference(&config.method);
    }

    fn parse_config(&self) -> Result<TreeInferenceConfig, Box<dyn Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: TreeInferenceConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    fn log_input(&self, config: &TreeInferenceConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.data.sample_counts);
        log::info!("{:18}: {}", "File counts", config.data.file_counts);
    }

    fn run_tree_inference(&self, methods: &[TreeInferenceMethod]) {
        if methods.len() > 1 || methods.is_empty() {
            self.infer_all_trees();
        } else {
            match methods[0] {
                TreeInferenceMethod::MLSpeciesTree => self.infer_ml_tree(),
                TreeInferenceMethod::MLGeneTree => self.infer_ml_gene_tree(),
                TreeInferenceMethod::GeneSiteConcordance => self.infer_gsc_tree(),
                TreeInferenceMethod::MSCSpeciesTree => self.infer_msc_tree(),
            }
        };
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
