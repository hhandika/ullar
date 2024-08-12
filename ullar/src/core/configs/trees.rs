use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    core::configs::CONFIG_EXTENSION, helper::files::FileMetadata, types::alignments::AlignmentFiles,
};

pub const DEFAULT_TREE_PREFIX: &str = "tree";

pub const DEFAULT_TREE_CONFIG_DIR: &str = "configs";

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeInferenceConfig {
    pub input_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trees: Option<TreeFiles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignments: Option<AlignmentFiles>,
}

impl Default for TreeInferenceConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            trees: None,
            alignments: None,
        }
    }
}

impl TreeInferenceConfig {
    pub fn new(
        input_dir: &Path,
        trees: Option<TreeFiles>,
        alignments: Option<AlignmentFiles>,
    ) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            trees,
            alignments,
        }
    }

    /// Serialize the configuration to a YAML file.
    pub fn to_yaml(&self, output_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        fs::create_dir_all(output_dir)?;
        let mut output = output_dir.join("tree_inference_config");
        output.set_extension(CONFIG_EXTENSION);
        let writer = File::create(&output)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeFiles {
    pub tip_counts: Option<usize>,
    pub trees: Vec<FileMetadata>,
}

impl TreeFiles {
    pub fn new(tip_counts: Option<usize>, trees: Vec<FileMetadata>) -> Self {
        Self { tip_counts, trees }
    }
}
