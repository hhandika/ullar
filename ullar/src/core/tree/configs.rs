use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    cli::commands::tree::IqTreeSettingArgs,
    core::deps::{
        iqtree::IqtreeMetadata,
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

pub const MSC_INFERENCE_DEP_NAME: &str = "msc_inference";
pub const TREE_INFERENCE_DEP_NAME: &str = "tree_inference";
pub const DATA_PREPARATION_DEP_NAME: &str = "data_preparation";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceConfig {
    #[serde(flatten)]
    pub app: UllarConfig,
    pub input: TreeInferenceInput,
    pub iqtree_config: Option<IQTreeConfig>,
    pub dependencies: BTreeMap<String, DepMetadata>,
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
            dependencies: BTreeMap::new(),
            alignments,
            iqtree_config: None,
        }
    }

    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = std::fs::read_to_string(config_path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_toml(&mut self) -> Result<PathBuf, Box<dyn Error>> {
        self.get_metadata();
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(&output_path, toml)?;
        Ok(output_path)
    }

    #[deprecated(since = "0.5.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&mut self) -> Result<PathBuf, Box<dyn Error>> {
        self.get_iqtree_metadata();
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn get_metadata(&mut self) {
        self.get_segul_metadata();
        let used_iqtree = self.input.methods.iter().find(|m| {
            matches!(
                m,
                TreeInferenceMethod::MlSpeciesTree
                    | TreeInferenceMethod::MlGeneTree
                    | TreeInferenceMethod::GeneSiteConcordance
            )
        });
        let used_aster = self
            .input
            .methods
            .iter()
            .find(|m| matches!(m, TreeInferenceMethod::MscSpeciesTree));
        if used_iqtree.is_some() {
            let meta = self.get_iqtree_metadata();
            self.dependencies
                .insert(TREE_INFERENCE_DEP_NAME.to_string(), meta);
        }
        if used_aster.is_some() {
            let meta = self.get_msc_inference_metadata();
            self.dependencies
                .insert(MSC_INFERENCE_DEP_NAME.to_string(), meta);
        }
    }

    fn get_msc_inference_metadata(&mut self) -> DepMetadata {
        DepMetadata::new("msc", "TEST", None)
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

    fn get_segul_metadata(&mut self) {
        let methods = [
            SegulMethods::AlignmentFinding,
            SegulMethods::AlignmentConcatenation,
            SegulMethods::AlignmentSummary,
        ];
        let methods_str: Vec<String> = methods.iter().map(|m| m.as_str().to_string()).collect();
        let dep = get_segul_metadata().with_methods(methods_str);
        self.dependencies
            .insert(DATA_PREPARATION_DEP_NAME.to_string(), dep);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceInput {
    pub input_dir: PathBuf,
    pub methods: Vec<TreeInferenceMethod>,
}

impl TreeInferenceInput {
    pub fn new(input_dir: &Path, methods: Vec<TreeInferenceMethod>) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            methods,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IQTreeConfig {
    pub partition_model: String,
    pub models: String,
    pub threads: String,
    pub bootstrap: String,
    pub override_args_species: Option<String>,
    pub override_args_genes: Option<String>,
}

impl IQTreeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &IqTreeSettingArgs) -> Self {
        Self {
            partition_model: args.partition_model.to_string(),
            models: args.models.to_string(),
            threads: args.threads.to_string(),
            bootstrap: args.bootstrap.to_string(),
            override_args_species: args.override_args_species.clone(),
            override_args_genes: args.override_args_genes.clone(),
        }
    }
}
