use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    cli::commands::tree::IqTreeSettingArgs,
    core::deps::{
        iqtree::{IqtreeMetadata, DEFAULT_IQTREE_MODEL, DEFAULT_IQTREE_THREADS},
        segul::{get_segul_metadata, SegulMethods},
    },
    helper::common::UllarConfig,
    types::alignments::AlignmentFiles,
};
use crate::{
    core::deps::DepMetadata, helper::configs::generate_config_output_path,
    types::TreeInferenceMethod,
};

pub const DEFAULT_TREE_PREFIX: &str = "tree";
pub const DEFAULT_ML_INFERENCE_CONFIG: &str = "ml_inference";

pub const SPECIES_TREE_ANALYSIS: &str = "species_tree_inference";
pub const GENE_TREE_ANALYSIS: &str = "gene_tree_inference";
pub const MSC_INFERENCE_DEP_NAME: &str = "msc_inference";
pub const DATA_PREPARATION_DEP_NAME: &str = "data_preparation";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceConfig {
    #[serde(flatten)]
    pub app: UllarConfig,
    pub input: TreeInferenceInput,
    pub data_preparation: DepMetadata,
    pub analyses: BTreeMap<String, TreeInferenceAnalyses>,
    pub alignments: AlignmentFiles,
}

impl TreeInferenceConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(
        input_dir: &Path,
        methods: &[TreeInferenceMethod],
        alignments: AlignmentFiles,
    ) -> Self {
        Self {
            app: UllarConfig::default(),
            input: TreeInferenceInput::new(input_dir, methods.to_vec()),
            data_preparation: get_segul_metadata(),
            analyses: BTreeMap::new(),
            alignments,
        }
    }

    pub fn set_species_tree_params(&mut self, args: &IqTreeSettingArgs) {
        let dependency = self.get_iqtree_metadata();
        let params = TreeInferenceAnalyses::new(dependency).set_species_tree_params(args);
        self.analyses
            .insert(SPECIES_TREE_ANALYSIS.to_string(), params);
    }

    pub fn set_gene_tree_params(&mut self, args: &IqTreeSettingArgs) {
        let dependency = self.get_iqtree_metadata();
        let mut params = TreeInferenceAnalyses::new(dependency);
        params.set_gene_tree_params(args);
        self.analyses.insert(GENE_TREE_ANALYSIS.to_string(), params);
    }

    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = std::fs::read_to_string(config_path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_toml(&mut self) -> Result<PathBuf, Box<dyn Error>> {
        self.update_segul_metadata();
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(&output_path, toml)?;
        Ok(output_path)
    }

    #[deprecated(since = "0.5.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&mut self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    #[allow(dead_code)]
    fn get_msc_inference_metadata(&mut self, methods: Vec<String>) -> DepMetadata {
        DepMetadata::new("msc", "TEST", None).with_methods(methods)
    }

    fn get_iqtree_metadata(&self) -> DepMetadata {
        let dep = IqtreeMetadata::new().get();

        match dep {
            Some(metadata) => metadata,
            None => {
                panic!(
                    "IQ-TREE not found. Please, install it first. \
                ULLAR can use either iqtree v1 or v2. \
                It will prioritize iqtree2 if both are installed."
                );
            }
        }
    }

    fn update_segul_metadata(&mut self) {
        let methods = [
            SegulMethods::AlignmentFinding,
            SegulMethods::AlignmentConcatenation,
            SegulMethods::AlignmentSummary,
        ];
        let methods_str: Vec<String> = methods.iter().map(|m| m.as_str().to_string()).collect();
        self.data_preparation.set_methods(methods_str);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TreeInferenceInput {
    pub input_dir: PathBuf,
    pub analysis_summary: Vec<TreeInferenceMethod>,
}

impl TreeInferenceInput {
    pub fn new(input_dir: &Path, analysis_summary: Vec<TreeInferenceMethod>) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            analysis_summary,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceAnalyses {
    pub dependency: DepMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species_tree_params: Option<IqTreeParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene_tree_params: Option<IqTreeParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msc_tree_params: Option<IqTreeParams>,
}

impl TreeInferenceAnalyses {
    pub fn new(dependency: DepMetadata) -> Self {
        Self {
            dependency,
            species_tree_params: None,
            gene_tree_params: None,
            msc_tree_params: None,
        }
    }

    pub fn set_species_tree_params(mut self, args: &IqTreeSettingArgs) -> Self {
        match &args.override_args_species {
            Some(arg) => {
                let mut params = IqTreeParams::new();
                params.override_params(arg);
                self.species_tree_params = Some(params);
                self
            }
            None => {
                let params = IqTreeParams::from_args(args)
                    .with_optional_args(args.optional_args_species.as_deref());
                self.species_tree_params = Some(params);
                self
            }
        }
    }

    pub fn set_gene_tree_params(&mut self, args: &IqTreeSettingArgs) {
        match &args.override_args_genes {
            Some(arg) => {
                let mut params = IqTreeParams::new();
                params.override_params(arg);
                self.gene_tree_params = Some(params);
            }
            None => {
                let params = IqTreeParams::from_args(args)
                    .with_optional_args(args.optional_args_genes.as_deref());
                self.gene_tree_params = Some(params);
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IqTreeParams {
    pub partition_model: String,
    pub models: String,
    pub threads: String,
    pub bootstrap: Option<String>,
    pub optional_args: Option<String>,
}

impl IqTreeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &IqTreeSettingArgs) -> Self {
        Self {
            partition_model: args.partition_model.to_string(),
            models: args.models.to_string(),
            threads: args.threads.to_string(),
            bootstrap: args.bootstrap.clone(),
            optional_args: None,
        }
    }

    pub fn with_optional_args(mut self, args: Option<&str>) -> Self {
        self.optional_args = args.map(|a| a.to_string());
        self
    }

    pub fn override_params(&mut self, args: &str) {
        let mut params = args.to_string();
        self.models = self.capture_models(&mut params);
        self.threads = self.capture_threads(&mut params);
        self.bootstrap = self.capture_bs_value(&mut params);
        self.optional_args = Some(args.to_string());
    }

    fn capture_models(&self, params: &mut String) -> String {
        let re = Regex::new(r"(?<models>-m)\s+(?<value>\S+)").expect("Failed to compile regex");
        let capture = re.captures(params).expect("Failed to capture models");
        match capture.name("value") {
            Some(v) => {
                let value = v.as_str().to_string();
                let model = format!("{} {}", capture.name("models").unwrap().as_str(), value);
                *params = params.replace(&model, "");
                value
            }
            None => DEFAULT_IQTREE_MODEL.to_string(),
        }
    }

    fn capture_bs_value(&self, params: &mut String) -> Option<String> {
        let re = Regex::new(r"(?<bs>-B|b)\s+(?<value>\d+)").expect("Failed to compile regex");
        let bs = re
            .captures(params)
            .expect("Failed to capture bootstrap value");
        match bs.name("value") {
            Some(v) => {
                let value = v.as_str().to_string();
                let arg = format!("{} {}", bs.name("bs").unwrap().as_str(), value);
                *params = params.replace(&arg, "");
                Some(value)
            }
            None => None,
        }
    }

    fn capture_threads(&self, params: &mut String) -> String {
        let re = Regex::new(r"(?<threads>-T|t)\s+(?<value>\d+)").expect("Failed to compile regex");
        let thread = re.captures(params).expect("Failed to capture thread value");
        match thread.name("value") {
            Some(v) => {
                let value = v.as_str().to_string();
                let arg = format!("{} {}", thread.name("threads").unwrap().as_str(), value);
                *params = params.replace(&arg, "");
                value
            }
            None => DEFAULT_IQTREE_THREADS.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! init {
        ($iqtree: ident) => {
            let $iqtree = IqTreeParams::new();
        };
    }

    #[test]
    fn test_bootstrap_value() {
        init!(iqtree);
        let mut params = String::from("-b 1000");
        let bs = iqtree.capture_bs_value(&mut params);
        assert_eq!(bs, Some(String::from("1000")));
    }

    #[test]
    fn test_threads_value() {
        init!(iqtree);
        let mut params = String::from("-T 4");
        let threads = iqtree.capture_threads(&mut params);
        assert_eq!(threads, "4");
    }
}
