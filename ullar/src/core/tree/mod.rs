use std::{
    error::Error,
    path::{Path, PathBuf},
};

use anyhow::Context;
use aster::MscAster;
use colored::Colorize;
use configs::{
    AsterParams, IqTreeParams, TreeInferenceConfig, DEFAULT_ML_INFERENCE_CONFIG,
    GENE_SITE_CONCORDANCE_ANALYSIS, GENE_TREE_ANALYSIS, MSC_INFERENCE_ANALYSIS,
    SPECIES_TREE_ANALYSIS,
};
use iqtree::{GeneSiteConcordance, IQTreeResults, MlGeneTree, MlSpeciesTree};

use crate::{
    cli::commands::tree::TreeInferenceArgs,
    helper::{
        common::{self, PrettyHeader},
        configs::{CONFIG_EXTENSION_TOML, DEFAULT_CONFIG_DIR},
        files::PathCheck,
    },
    types::{runner::RunnerOptions, trees::TreeInferenceMethod},
};

use super::deps::{aster::AsterMetadata, iqtree::IqtreeMetadata};

pub mod aster;
pub mod configs;
pub mod init;
pub mod iqtree;

pub const DEFAULT_PHYLO_OUTPUT_DIR: &str = "out_trees";

pub const DEFAULT_ML_SPECIES_TREE_OUTPUT_DIR: &str = "ml_species_tree";
pub const DEFAULT_ML_GENE_TREE_OUTPUT_DIR: &str = "ml_gene_trees";
pub const DEFAULT_GSC_OUTPUT_DIR: &str = "gsc_trees";
pub const DEFAULT_MSC_ASTRAL_OUTPUT_DIR: &str = "msc_astral_trees";
pub const DEFAULT_MSC_ASTRAL_PRO_OUTPUT_DIR: &str = "msc_astral_pro_trees";
pub const DEFAULT_MSC_WASTRAL_OUTPUT_DIR: &str = "msc_wastral_trees";

pub struct TreeEstimation<'a> {
    /// Path to raw read config file
    pub config_path: PathBuf,
    /// Parent output directory
    pub output_dir: &'a Path,
    /// Runner options
    pub runner: RunnerOptions,
}

impl<'a> TreeEstimation<'a> {
    /// Initialize a new TreeEstimation instance
    /// with the given parameters
    pub fn new<P: AsRef<Path>>(config_path: P, output_dir: &'a Path) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            output_dir,
            runner: RunnerOptions::default(),
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
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
        }
    }

    /// Initialize a new TreeEstimation instance
    /// from a given config path and output directory
    ///
    pub fn from_config_path<P: AsRef<Path>>(config_path: P, output_dir: &'a Path) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            output_dir,
            runner: RunnerOptions::default(),
        }
    }

    pub fn infer(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Parsing and checking the config file\n");
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
        self.check_path_exists(&config.input.analyses);
        self.run_tree_inference(&config)
            .expect("Failed to run tree inference");
    }

    fn run_tree_inference(&self, config: &TreeInferenceConfig) -> Result<(), Box<dyn Error>> {
        let mut iqtree_results = IQTreeResults::new();
        for analysis in &config.input.analyses {
            self.print_header(analysis);
            let output_dir = self.generate_output_path(analysis);
            match analysis {
                TreeInferenceMethod::MlSpeciesTree => {
                    self.infer_ml_species_tree(config, &mut iqtree_results, &output_dir)?
                }

                TreeInferenceMethod::MlGeneTree => {
                    self.infer_ml_gene_trees(config, &mut iqtree_results, &output_dir)?
                }
                TreeInferenceMethod::GeneSiteConcordance => {
                    self.infer_concordance_factor(config, &mut iqtree_results, &output_dir)?
                }
                TreeInferenceMethod::MscSpeciesTree => {
                    self.infer_msc_trees(config, &mut iqtree_results, &output_dir)?
                }
            }
        }
        Ok(())
    }

    fn generate_output_path(&self, analysis: &TreeInferenceMethod) -> PathBuf {
        match analysis {
            TreeInferenceMethod::MlSpeciesTree => {
                self.output_dir.join(DEFAULT_ML_SPECIES_TREE_OUTPUT_DIR)
            }
            TreeInferenceMethod::MlGeneTree => {
                self.output_dir.join(DEFAULT_ML_GENE_TREE_OUTPUT_DIR)
            }
            TreeInferenceMethod::GeneSiteConcordance => {
                self.output_dir.join(DEFAULT_GSC_OUTPUT_DIR)
            }
            TreeInferenceMethod::MscSpeciesTree => {
                self.output_dir.join(DEFAULT_MSC_ASTRAL_OUTPUT_DIR)
            }
        }
    }

    fn check_path_exists(&self, analysis: &[TreeInferenceMethod]) {
        analysis.iter().for_each(|a| {
            log::info!("Checking output directory for {}", a.to_string());
            let output_dir = self.generate_output_path(a);
            PathCheck::new(&output_dir).is_dir().prompt_exists(false);
        });
    }

    fn infer_ml_species_tree(
        &self,
        config: &TreeInferenceConfig,
        iqtree_result: &mut IQTreeResults,
        output_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let dep = config.analyses.get(SPECIES_TREE_ANALYSIS);

        match dep {
            Some(d) => {
                let prefix = "concat";
                let params = d
                    .species_tree_params
                    .as_ref()
                    .with_context(|| "Species tree parameters not found")?;
                self.log_iqtree(params);
                let ml_analyses = MlSpeciesTree::new(&config.alignments, &params, &output_dir);
                ml_analyses.infer_species_tree(iqtree_result, prefix)?;
                self.log_output(&output_dir);
                log::info!(
                    "{:18}: {}",
                    "Treefile",
                    iqtree_result.species_tree.display()
                );
                Ok(())
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

    fn infer_ml_gene_trees(
        &self,
        config: &TreeInferenceConfig,
        iqtree_result: &mut IQTreeResults,
        output_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let dep = config.analyses.get(GENE_TREE_ANALYSIS);
        match dep {
            Some(d) => {
                let params = d
                    .gene_tree_params
                    .as_ref()
                    .with_context(|| "Gene tree parameters not found")?;
                self.log_iqtree(params);
                let ml_analyses = MlGeneTree::new(&config.alignments, &params, &output_dir);
                ml_analyses.infer_gene_trees(iqtree_result);
                self.log_output(&output_dir);
                log::info!("{:18}: {}", "Treefile", iqtree_result.gene_trees.display());
                Ok(())
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

    fn infer_concordance_factor(
        &self,
        config: &TreeInferenceConfig,
        iqtree_results: &mut IQTreeResults,
        output_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let dep = config.analyses.get(GENE_SITE_CONCORDANCE_ANALYSIS);
        match dep {
            Some(d) => {
                let params = d
                    .concordance_factor
                    .as_ref()
                    .with_context(|| "Gene tree parameters not found.")?;
                self.log_iqtree(params);
                let ml_analyses = GeneSiteConcordance::new(&params, &output_dir);
                ml_analyses.infer_concordance_factor(iqtree_results)?;
                self.log_output(&output_dir);
                Ok(())
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

    fn infer_msc_trees(
        &self,
        config: &TreeInferenceConfig,
        iqtree_results: &mut IQTreeResults,
        output_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let dep = config.analyses.get(MSC_INFERENCE_ANALYSIS);
        match dep {
            Some(d) => {
                let params = d
                    .msc_methods
                    .as_ref()
                    .with_context(|| "MSC parameters not found")?;
                self.log_msc_inference(params, iqtree_results);
                let msc_analyses = MscAster::new(&params, &iqtree_results.gene_trees, &output_dir);
                msc_analyses.infer();
                self.log_output(&output_dir);
                Ok(())
            }
            None => {
                let error = format!(
                    "{} No MSC analysis specified in the config files. Skipping",
                    "Warning:".yellow()
                );
                Err(error.into())
            }
        }
    }

    fn print_header(&self, analysis: &TreeInferenceMethod) {
        let mut decorator = PrettyHeader::new();
        let header = decorator.get_section_header(&analysis.to_string());
        log::info!("{}", header);
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
        if required_v2 && !iqtree_version.starts_with("2") {
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
        for method in aster_params.methods.keys() {
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

    fn log_input(&self, config: &TreeInferenceConfig) {
        log::info!("\n{}", "Input".cyan());
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        log::info!(
            "{:18}: {}",
            "Sample counts",
            config.alignments.sample_counts
        );
        log::info!(
            "{:18}: {}\n\n",
            "File counts",
            config.alignments.file_counts
        );
    }

    fn log_iqtree(&self, params: &IqTreeParams) {
        let not_found = "Not found".red();
        match &params.dependency {
            Some(dep) => {
                log::info!("{:18}: {}", "App", "IQ-TREE");
                log::info!("{:18}: {}", "Version", dep.version);
                log::info!(
                    "{:18}: {}\n",
                    "Executable",
                    dep.executable
                        .as_ref()
                        .unwrap_or(&not_found.to_string())
                        .as_str()
                );
            }
            None => {
                log::info!("{:18}: {}", "App", "IQ-TREE".cyan());
            }
        }

        log::info!("{}", "Parameters".cyan());
        log::info!("{:18}: {}", "Subs. model", params.models);
        if let Some(partition) = &params.partition_model {
            log::info!("{:18}: {}", "Partition model", partition.to_string());
        }
        log::info!("{:18}: {}", "Threads", params.threads);
        if let Some(bootstrap) = &params.bootstrap {
            log::info!("{:18}: {}", "Bootstrap", bootstrap);
        }
        if let Some(optional) = &params.optional_args {
            log::info!("{:18}: {}", "Additional", optional);
        }
        log::info!("");
    }

    fn log_msc_inference(&self, params: &AsterParams, trees: &IQTreeResults) {
        log::info!("{:18}: {}", "App", "ASTER");
        params.methods.iter().for_each(|(method, dep)| {
            log::info!("{:18}: {}", "Method", method);
            match dep {
                Some(d) => {
                    log::info!("{:18}: {}", "Version", d.version);
                    log::info!(
                        "{:18}: {}",
                        "Executable",
                        d.executable.as_ref().unwrap_or(&"Not found".to_string())
                    );
                }
                None => {
                    log::info!("{:18}: {}", "Executable", "Not found".red());
                }
            }
        });

        log::info!("\n{}", "Input".cyan());
        log::info!("{:18}: {}", "Gene trees", trees.gene_trees.display());
        if let Some(opts) = &params.optional_args {
            log::info!("{:18}: {}", "Optional args", opts);
        }
        log::info!("");
    }

    fn log_output(&self, output_dir: &Path) {
        log::info!("\n{}", "Output".cyan());
        log::info!("{:18}: {}", "Output directory", output_dir.display());
    }
}
