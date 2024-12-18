use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{core::deps::iqtree::IqtreeMetadata, types::alignments::AlignmentFiles};
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
    pub alignments: AlignmentFiles,
}

impl TreeInferenceConfig {
    pub fn new(
        input_dir: &Path,
        methods: Vec<TreeInferenceMethod>,
        alignments: AlignmentFiles,
    ) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            methods,
            dependencies: Vec::new(),
            alignments,
        }
    }

    pub fn to_yaml(&mut self, override_args: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
        self.get_dependency(override_args);
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn get_dependency(&mut self, override_args: Option<&str>) {
        let dep = IqtreeMetadata::new(override_args).get();

        match dep {
            Some(metadata) => self.dependencies = vec![metadata],
            None => {
                panic!(
                    "IQ-TREE not found. Please, install it first. \
                ULLAR can use either iqtree v1 or v2. \
                And will prioritize iqtree2 if both are installed."
                );
            }
        }
    }
}
