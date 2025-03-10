use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    cli::commands::tree::{AsterSettingArgs, IqTreeSettingArgs},
    core::deps::{
        aster::AsterMetadata,
        iqtree::{IqtreeMetadata, DEFAULT_IQTREE_MODEL, DEFAULT_IQTREE_THREADS},
        segul::{get_segul_metadata, SegulMethods},
    },
    helper::common::UllarConfig,
    types::{alignments::AlignmentFiles, trees::MscInferenceMethod},
};
use crate::{
    core::deps::DepMetadata, helper::configs::generate_config_output_path,
    types::trees::TreeInferenceMethod,
};

pub const DEFAULT_TREE_PREFIX: &str = "tree";
pub const DEFAULT_ML_INFERENCE_CONFIG: &str = "ml_inference";

pub const SPECIES_TREE_ANALYSIS: &str = "species_tree_inference";
pub const GENE_TREE_ANALYSIS: &str = "gene_tree_inference";
pub const GENE_SITE_CONCORDANCE_ANALYSIS: &str = "gene_site_concordance";
pub const MSC_INFERENCE_ANALYSIS: &str = "msc_inference";
pub const DATA_PREPARATION_DEP_NAME: &str = "data_preparation";

/// We need to reorder the analyses to ensure
/// that species tree and gene tree are inferred first.
/// The rest can be inferred in any order.
pub fn reorder_analyses(analyses: &mut Vec<TreeInferenceMethod>) {
    let mut reorder_analyses = Vec::with_capacity(analyses.len());
    let ml_species = analyses
        .iter()
        .position(|a| a == &TreeInferenceMethod::MlSpeciesTree);
    if let Some(index) = ml_species {
        let analysis = analyses.remove(index);
        reorder_analyses.push(analysis);
    }

    let ml_gene = analyses
        .iter()
        .position(|a| a == &TreeInferenceMethod::MlGeneTree);

    if let Some(index) = ml_gene {
        let analysis = analyses.remove(index);
        reorder_analyses.push(analysis);
    }
    reorder_analyses.extend(analyses.drain(..));
    *analyses = reorder_analyses;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceConfig {
    #[serde(flatten)]
    pub app: UllarConfig,
    pub input: TreeInferenceInput,
    pub data_preparation: DepMetadata,
    // We use an IndexMap instead of BTreeMap
    // or HashMap to maintain the order of analyses.
    pub analyses: IndexMap<String, TreeInferenceAnalyses>,
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
            app: UllarConfig::init(),
            input: TreeInferenceInput::new(input_dir, methods.to_vec()),
            data_preparation: get_segul_metadata(),
            analyses: IndexMap::new(),
            alignments,
        }
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

    pub fn update_analyses(
        &mut self,
        analyses: &[TreeInferenceMethod],
        iqtree_args: &IqTreeSettingArgs,
        aster_args: &AsterSettingArgs,
    ) {
        for analysis in analyses {
            match analysis {
                TreeInferenceMethod::MlSpeciesTree => self.set_species_tree_params(iqtree_args),
                TreeInferenceMethod::MlGeneTree => self.set_gene_tree_params(iqtree_args),
                TreeInferenceMethod::GeneSiteConcordance => {
                    self.set_concordance_factor_params(iqtree_args)
                }
                TreeInferenceMethod::MscSpeciesTree => self.set_msc_params(aster_args),
            }
        }
    }

    pub fn has_ml_species_tree(&self) -> bool {
        self.input
            .analyses
            .iter()
            .any(|a| a == &TreeInferenceMethod::MlSpeciesTree)
    }

    pub fn has_ml_gene_tree(&self) -> bool {
        self.input
            .analyses
            .iter()
            .any(|a| a == &TreeInferenceMethod::MlGeneTree)
    }

    pub fn has_msc(&self) -> bool {
        self.input
            .analyses
            .iter()
            .any(|a| a == &TreeInferenceMethod::MscSpeciesTree)
    }

    #[deprecated(since = "0.5.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&mut self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn set_species_tree_params(&mut self, args: &IqTreeSettingArgs) {
        let params = TreeInferenceAnalyses::new().set_species_tree_params(args);
        self.analyses
            .insert(SPECIES_TREE_ANALYSIS.to_string(), params);
    }

    fn set_gene_tree_params(&mut self, args: &IqTreeSettingArgs) {
        let mut params = TreeInferenceAnalyses::new();
        params.set_gene_tree_params(args);
        self.analyses.insert(GENE_TREE_ANALYSIS.to_string(), params);
    }

    fn set_concordance_factor_params(&mut self, args: &IqTreeSettingArgs) {
        let mut params = TreeInferenceAnalyses::new();
        params.set_concordance_factor_params(args);
        self.analyses
            .insert(GENE_SITE_CONCORDANCE_ANALYSIS.to_string(), params);
    }

    fn set_msc_params(&mut self, args: &AsterSettingArgs) {
        let mut params = TreeInferenceAnalyses::new();
        params.set_msc_methods(args);
        self.analyses
            .insert(MSC_INFERENCE_ANALYSIS.to_string(), params);
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
    pub analyses: Vec<TreeInferenceMethod>,
}

impl TreeInferenceInput {
    pub fn new(input_dir: &Path, analyses: Vec<TreeInferenceMethod>) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            analyses,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceAnalyses {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub species_tree_params: Option<IqTreeParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub gene_tree_params: Option<IqTreeParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub concordance_factor: Option<IqTreeParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub msc_methods: Option<AsterParams>,
}

impl TreeInferenceAnalyses {
    pub fn new() -> Self {
        Self {
            species_tree_params: None,
            gene_tree_params: None,
            concordance_factor: None,
            msc_methods: None,
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

    pub fn set_concordance_factor_params(&mut self, args: &IqTreeSettingArgs) {
        let params =
            IqTreeParams::from_args(args).with_optional_args(args.optional_args_gscf.as_deref());
        self.concordance_factor = Some(params);
    }

    pub fn set_msc_methods(&mut self, args: &AsterSettingArgs) {
        let params =
            AsterParams::from_args(args).with_optional_args(args.optional_args_msc.as_deref());
        self.msc_methods = Some(params);
    }
}

// Include all ASTER software suites.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AsterParams {
    pub dependency: Vec<Option<DepMetadata>>,
    pub methods: Vec<MscInferenceMethod>,
    pub optional_args: Option<String>,
}

impl AsterParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &AsterSettingArgs) -> Self {
        let methods = match &args.specify_msc_methods {
            Some(methods) => methods
                .iter()
                .map(|m| m.parse::<MscInferenceMethod>().expect("Invalid method"))
                .collect(),
            None => vec![MscInferenceMethod::default()],
        };
        Self {
            dependency: methods
                .iter()
                .map(|m| AsterMetadata::new().get_matching(m))
                .collect(),
            methods,
            optional_args: args.optional_args_msc.clone(),
        }
    }

    pub fn with_optional_args(mut self, args: Option<&str>) -> Self {
        self.optional_args = args.map(|a| a.to_string());
        self
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IqTreeParams {
    #[serde(flatten)]
    pub dependency: Option<DepMetadata>,
    pub partition_model: String,
    pub models: String,
    pub threads: String,
    pub bootstrap: Option<String>,
    pub optional_args: Option<String>,
    // Only used for gene site concordance factor
    pub recompute_likelihoods: bool,
}

impl IqTreeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &IqTreeSettingArgs) -> Self {
        Self {
            dependency: IqtreeMetadata::new().get(),
            partition_model: args.partition_model.to_string(),
            models: args.models.to_string(),
            threads: args.threads.to_string(),
            bootstrap: args.bootstrap.clone(),
            optional_args: None,
            recompute_likelihoods: args.recompute_likelihoods,
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
                // This approach is simple, but will require memory allocation.
                // It will be a minor issue because the string input will be small.
                // A better, also simple, option without memory allocation is available
                // in the nightly Rust. We should switch to it when it becomes stable.
                // params.remove_matches(&arg);
                // https://doc.rust-lang.org/std/string/struct.String.html#method.remove_matches
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
