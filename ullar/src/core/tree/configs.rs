use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{core::deps::DepMetadata, helper::configs::generate_config_output_path};
use crate::{helper::files::FileMetadata, types::alignments::AlignmentFiles};

pub const DEFAULT_TREE_PREFIX: &str = "tree";
pub const DEFAULT_ML_INFERENCE_CONFIG: &str = "ml_inference";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeInferenceConfig {
    pub config_path: Option<PathBuf>,
    pub input_init_dir: PathBuf,
    pub sample_counts: usize,
    pub file_counts: usize,
    pub cleaned: bool,
    pub dependencies: Vec<DepMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    pub alignments: AlignmentFiles,
}

impl TreeInferenceConfig {
    pub fn new(
        config_path: Option<PathBuf>,
        input_init_dir: &Path,
        cleaned: bool,
        dependencies: Vec<DepMetadata>,
        override_args: Option<String>,
        alignments: AlignmentFiles,
    ) -> Self {
        Self {
            config_path,
            input_init_dir: input_init_dir.to_path_buf(),
            sample_counts: 0,
            file_counts: 0,
            cleaned,
            dependencies,
            override_args,
            alignments,
        }
    }

    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_ML_INFERENCE_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct TreeInferenceConfig {
//     pub input_dir: PathBuf,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub trees: Option<TreeFiles>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub alignments: Option<AlignmentFiles>,
// }

// impl Default for TreeInferenceConfig {
//     fn default() -> Self {
//         Self {
//             input_dir: PathBuf::new(),
//             trees: None,
//             alignments: None,
//         }
//     }
// }

// impl TreeInferenceConfig {
//     pub fn new(
//         input_dir: &Path,
//         trees: Option<TreeFiles>,
//         alignments: Option<AlignmentFiles>,
//     ) -> Self {
//         Self {
//             input_dir: input_dir.to_path_buf(),
//             trees,
//             alignments,
//         }
//     }

//     /// Serialize the config to a YAML file.
//     pub fn to_yaml(&self, output_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
//         fs::create_dir_all(output_dir)?;
//         let mut output = output_dir.join("tree_inference_config");
//         output.set_extension(CONFIG_EXTENSION);
//         let writer = File::create(&output)?;
//         serde_yaml::to_writer(&writer, self)?;
//         Ok(output)
//     }
// }

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
