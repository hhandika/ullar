use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;
use configs::{TreeInferenceConfig, DEFAULT_ML_INFERENCE_CONFIG, TREE_INFERENCE_DEP_NAME};
use iqtree::MlSpeciesTree;

use crate::{
    cli::commands::tree::TreeInferenceArgs,
    helper::{
        common,
        configs::{CONFIG_EXTENSION_TOML, DEFAULT_CONFIG_DIR},
    },
    types::{runner::RunnerOptions, Task, TreeInferenceMethod},
};

use crate::core::deps::DepMetadata;

use super::deps::iqtree::IqtreeMetadata;

pub mod configs;
pub mod init;
pub mod iqtree;

pub const DEFAULT_PHYLO_OUTPUT_DIR: &str = "phylogenetic_tree";
pub const DEFAULT_ML_OUTPUT_DIR: &str = "ml_iqtree";
pub const DEFAULT_MSC_OUTPUT_DIR: &str = "msc_aster";

pub struct TreeEstimation<'a> {
    /// Path to raw read config file
    pub config_path: PathBuf,
    /// Checksum verification flag
    pub ignore_checksum: bool,
    /// Parent output directory
    pub output_dir: &'a Path,
    /// Runner options
    pub runner: RunnerOptions,
    #[allow(dead_code)]
    task: Task,
}

impl<'a> TreeEstimation<'a> {
    /// Initialize a new TreeEstimation instance
    /// with the given parameters
    pub fn new<P: AsRef<Path>>(
        config_path: P,
        ignore_checksum: bool,
        output_dir: &'a Path,
    ) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            ignore_checksum,
            output_dir,
            runner: RunnerOptions::default(),
            task: Task::TreeInference,
        }
    }

    /// Initialize a new TreeEstimation instance
    /// from command line arguments
    pub fn from_arg(args: &'a TreeInferenceArgs) -> Self {
        let config_path = match &args.config {
            Some(path) => path.to_owned(),
            None => PathBuf::from(DEFAULT_CONFIG_DIR)
                .join(DEFAULT_ML_INFERENCE_CONFIG)
                .with_extension(CONFIG_EXTENSION_TOML),
        };
        Self {
            config_path,
            ignore_checksum: args.common.ignore_checksum,
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::TreeInference,
        }
    }

    pub fn infer(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        if config.input.methods.is_empty() {
            log::warn!(
                "{} No tree inference method specified in the config files. Using all methods",
                "Warning:".yellow()
            );
        }
        self.run_tree_inference(&config.input.methods, &config);
    }

    fn parse_config(&self) -> Result<TreeInferenceConfig, Box<dyn Error>> {
        let config: TreeInferenceConfig = TreeInferenceConfig::from_toml(&self.config_path)?;
        Ok(config)
    }

    fn log_input(&self, config: &TreeInferenceConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        log::info!(
            "{:18}: {}",
            "Sample counts",
            config.alignments.sample_counts
        );
        log::info!("{:18}: {}\n", "File counts", config.alignments.file_counts);
    }

    fn run_tree_inference(&self, methods: &[TreeInferenceMethod], config: &TreeInferenceConfig) {
        if methods.len() > 1 || methods.is_empty() {
            self.infer_all_trees(config);
        } else {
            match methods[0] {
                TreeInferenceMethod::MlSpeciesTree => self.infer_ml_tree(config),
                TreeInferenceMethod::MlGeneTree => self.infer_ml_gene_tree(),
                TreeInferenceMethod::GeneSiteConcordance => self.infer_gsc_tree(),
                TreeInferenceMethod::MscSpeciesTree => self.infer_msc_tree(),
            }
        };
    }

    fn infer_all_trees(&self, config: &TreeInferenceConfig) {
        self.infer_ml_tree(config);
        // self.infer_ml_gene_tree();
        // self.infer_gsc_tree();
        // self.infer_msc_tree();
    }

    fn infer_ml_tree(&self, config: &TreeInferenceConfig) {
        let prefix = "concat";
        let deps: Option<&DepMetadata> = config.dependencies.get(TREE_INFERENCE_DEP_NAME);

        if deps.is_none() {
            self.try_iqtree();
        }
        let iqtree = deps.expect("IQ-TREE dependency not found in the config");
        let ml_analyses = MlSpeciesTree::new(
            &config.alignments,
            &iqtree,
            &config.iqtree_config,
            &self.output_dir,
            prefix,
        );
        ml_analyses.infer(prefix);
    }

    fn try_iqtree(&self) -> DepMetadata {
        let dep = IqtreeMetadata::new().get();
        match dep {
            Some(d) => d,
            None => {
                log::error!("IQ-TREE dependency not found in the config");
                panic!("Exiting due to missing dependency");
            }
        }
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
