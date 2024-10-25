use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::types::alignments::AlignmentFiles;
use crate::{
    core::deps::DepMetadata, helper::configs::generate_config_output_path,
    types::TreeInferenceMethod,
};

pub const DEFAULT_TREE_PREFIX: &str = "tree";
pub const DEFAULT_ML_INFERENCE_CONFIG: &str = "ml_inference";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceConfig {
    pub input_dir: PathBuf,
    pub methods: Vec<TreeInferenceMethod>,
    pub dependencies: Vec<DepMetadata>,
    #[serde(flatten)]
    pub data: TreeData,
}

impl TreeInferenceConfig {
    pub fn new(input_dir: &Path, methods: Vec<TreeInferenceMethod>, data: TreeData) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            methods,
            dependencies: vec![],
            data,
        }
    }

    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }
}

#[derive(Debug, Serialize, Default, Deserialize)]
pub struct TreeData {
    pub sample_counts: usize,
    pub file_counts: usize,
    pub alignments: AlignmentFiles,
}

impl TreeData {
    pub fn new(alignments: AlignmentFiles) -> Self {
        Self {
            sample_counts: alignments.sample_counts,
            file_counts: alignments.file_counts,
            alignments,
        }
    }
}
