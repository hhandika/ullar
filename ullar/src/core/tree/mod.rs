use std::{
    error::Error,
    path::{Path, PathBuf},
};

use anyhow::Context;
use colored::Colorize;
use configs::{
    AsterParams, TreeInferenceConfig, DEFAULT_ML_INFERENCE_CONFIG, GENE_TREE_ANALYSIS,
    MSC_INFERENCE_ANALYSIS, SPECIES_TREE_ANALYSIS,
};
use iqtree::{IQTreeResults, MlSpeciesTree};

use crate::{
    cli::commands::tree::TreeInferenceArgs,
    helper::{
        common,
        configs::{CONFIG_EXTENSION_TOML, DEFAULT_CONFIG_DIR},
        files::PathCheck,
    },
    types::{runner::RunnerOptions, trees::TreeInferenceMethod, Task},
};

use super::deps::{aster::AsterMetadata, iqtree::IqtreeMetadata};

pub mod configs;
pub mod init;
pub mod iqtree;

pub const DEFAULT_PHYLO_OUTPUT_DIR: &str = "output_phylogenetic_inference";
pub const DEFAULT_ML_SPECIES_TREE_OUTPUT_DIR: &str = "ml_species_tree";
pub const DEFAULT_ML_GENE_TREE_OUTPUT_DIR: &str = "ml_gene_trees";
pub const DEFAULT_GSC_OUTPUT_DIR: &str = "gsc_trees";
pub const DEFAULT_MSC_ASTRAL_OUTPUT_DIR: &str = "msc_astral_trees";
pub const DEFAULT_MSC_ASTRAL_PRO_OUTPUT_DIR: &str = "msc_astral_pro_trees";
pub const DEFAULT_MSC_WASTRAL_OUTPUT_DIR: &str = "msc_wastral_trees";

const SPECIES_TREE_DIR: &str = "species_tree";

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
        let spinner = common::init_spinner();
        spinner.set_message("Parsing and checking the config file");
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        self.check_dependencies(&config)
            .expect("Failed finding dependencies");
        if config.input.analyses.is_empty() {
            log::error!(
                "{} No tree inference method specified in the config files.",
                "Warning:".red()
            );
            return;
        }
        spinner.finish_with_message("Skipping config data check\n");
        self.run_tree_inference(&config)
            .expect("Failed to run tree inference");
    }

    fn run_tree_inference(&self, config: &TreeInferenceConfig) -> Result<(), Box<dyn Error>> {
        let mut iqtree_results = IQTreeResults::new();
        for analysis in &config.input.analyses {
            match analysis {
                TreeInferenceMethod::MlSpeciesTree => {
                    let tree_path = self.infer_ml_species_tree(config)?;
                    iqtree_results.add_species_tree(tree_path);
                }
                TreeInferenceMethod::MlGeneTree => {
                    let tree_path = self.infer_ml_gene_trees(config)?;
                    iqtree_results.add_gene_trees(tree_path);
                }
                _ => unimplemented!("Tree inference method not implemented"),
            }
        }
        Ok(())
    }

    fn infer_ml_species_tree(
        &self,
        config: &TreeInferenceConfig,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let dep = config.analyses.get(SPECIES_TREE_ANALYSIS);
        match dep {
            Some(d) => {
                let output_dir = self.output_dir.join(SPECIES_TREE_DIR);
                PathCheck::new(&output_dir).is_dir().prompt_exists(false);
                let prefix = "concat";
                let params = d
                    .species_tree_params
                    .as_ref()
                    .with_context(|| "Species tree parameters not found")?;
                let ml_analyses = MlSpeciesTree::new(&config.alignments, &params, &output_dir);
                let tree_path = ml_analyses.infer_species_tree(prefix)?;
                Ok(tree_path)
            }
            None => {
                let error = format!(
                    "{} No ML species tree analysis specified in the config files. Skipping",
                    "Warning:".yellow()
                );
                Err(error.into())
            }
        }
    }

    fn infer_ml_gene_trees(&self, config: &TreeInferenceConfig) -> Result<PathBuf, Box<dyn Error>> {
        let dep = config.analyses.get(GENE_TREE_ANALYSIS);
        match dep {
            Some(d) => {
                let output_dir = self.output_dir.join(DEFAULT_ML_GENE_TREE_OUTPUT_DIR);
                PathCheck::new(&output_dir).is_dir().prompt_exists(false);
                let params = d
                    .gene_tree_params
                    .as_ref()
                    .with_context(|| "Gene tree parameters not found.")?;
                let ml_analyses = MlSpeciesTree::new(&config.alignments, &params, &output_dir);
                let tree_path = ml_analyses.infer_gene_trees();
                Ok(tree_path)
            }
            None => {
                let error = format!(
                    "{} No ML gene tree analysis specified in the config files. Skipping",
                    "Warning:".yellow()
                );
                Err(error.into())
            }
        }
    }

    // We check the dependency separately earlier
    // to warn the user before running the analysis
    fn check_dependencies(&self, config: &TreeInferenceConfig) -> Result<(), Box<dyn Error>> {
        self.check_iqtree_requirement(config)?;
        let required_aster = config
            .input
            .analyses
            .contains(&TreeInferenceMethod::MscSpeciesTree);
        if required_aster {
            match config.analyses.get(MSC_INFERENCE_ANALYSIS) {
                Some(d) => {
                    self.check_aster_requirement(d.msc_methods.as_ref())?;
                }
                None => {
                    let error = format!(
                        "{} No MSC analysis specified in the config files. Skipping",
                        "Warning:".yellow()
                    );
                    return Err(error.into());
                }
            }
        }
        Ok(())
    }

    fn check_iqtree_requirement(&self, config: &TreeInferenceConfig) -> Result<(), Box<dyn Error>> {
        let iqtree = IqtreeMetadata::new().get();
        if iqtree.is_none() {
            let error = format!(
                "{} IQ-TREE is not installed. \
                Please install IQ-TREE and update the config file.",
                "Error:".red()
            );
            return Err(error.into());
        }

        let iqtree_version = iqtree.expect("IQ-TREE dependency not found").version;
        let required_v2 = config
            .input
            .analyses
            .iter()
            .any(|f| *f == TreeInferenceMethod::GeneSiteConcordance);
        if required_v2 && iqtree_version.starts_with("2") {
            let error = format!(
                "{} IQ-TREE v2 is required for GSC analysis. \
                Please install IQ-TREE v2 and regenerate the config file.",
                "Error:".red()
            );
            return Err(error.into());
        }
        Ok(())
    }

    fn check_aster_requirement(&self, aster: Option<&AsterParams>) -> Result<(), Box<dyn Error>> {
        if aster.is_none() {
            let error = format!(
                "{} No MSC analysis specified in the config files. Skipping",
                "Warning:".yellow()
            );
            return Err(error.into());
        }
        let aster_params = aster.expect("MSC parameters not found in the config files");
        let mut missing = Vec::new();
        for method in &aster_params.methods {
            let dep = AsterMetadata::new().get_matching(method);
            if dep.is_none() {
                missing.push(method);
            }
        }
        missing.iter().for_each(|m| {
            let error = format!(
                "{} {} is not found. \
                Please ensure ASTER is installed and accessible in your PATH.\n\n",
                "Error:".red(),
                m.to_string()
            );
            log::error!("{}", error);
        });
        Ok(())
    }

    fn parse_config(&self) -> Result<TreeInferenceConfig, Box<dyn Error>> {
        let config: TreeInferenceConfig = TreeInferenceConfig::from_toml(&self.config_path)?;
        Ok(config)
    }

    #[allow(dead_code)]
    fn infer_ml_gene_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating ML gene tree");
        spinner.finish_with_message("Finished estimating ML gene tree\n");
    }

    // Gene Site Concordance Factor
    #[allow(dead_code)]
    fn infer_gsc_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating Gene Site Concordance Factor");
        spinner.finish_with_message("Finished estimating Gene Site Concordance Factor\n");
    }

    // Multi-Species Coalescent
    #[allow(dead_code)]
    fn infer_msc_tree(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Estimating MSC species tree");
        spinner.finish_with_message("Finished estimating MSC species tree\n");
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
}
