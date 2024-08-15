use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    core::utils::deps::DepMetadata, helper::configs::generate_config_output_path,
    types::alignments::AlignmentFiles,
};

/// Default alignment configuration straight from the aligner
pub const DEFAULT_RAW_ALIGNMENT_CONFIG: &str = "raw_alignment";

/// Default alignment configuration after cleaning
pub const DEFAULT_CLEANED_ALIGNMENT_CONFIG: &str = "cleaned_alignment";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlignmentConfig {
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

impl AlignmentConfig {
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
        let output_path = generate_config_output_path(self.get_config_filename());
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn get_config_filename(&self) -> &str {
        if self.cleaned {
            DEFAULT_CLEANED_ALIGNMENT_CONFIG
        } else {
            DEFAULT_RAW_ALIGNMENT_CONFIG
        }
    }
}
