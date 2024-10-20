use std::path::PathBuf;
use std::{error::Error, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::deps::fastp::FastpMetadata;
use crate::core::deps::DepMetadata;
use crate::helper::configs::generate_config_output_path;
use crate::helper::fastq::FastqConfigSummary;
use crate::types::reads::FastqReads;

pub const DEFAULT_READ_CLEANING_CONFIG: &str = "read_cleaning";

pub enum FileMatchingStrategy {
    Regex,
    CharacterSplit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanReadConfig {
    pub input_dir: PathBuf,
    #[serde(flatten)]
    pub input_summary: FastqConfigSummary,
    pub dependencies: DepMetadata,
    pub samples: Vec<FastqReads>,
}

impl Default for CleanReadConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            input_summary: FastqConfigSummary::default(),
            dependencies: DepMetadata::default(),
            samples: Vec::new(),
        }
    }
}

impl CleanReadConfig {
    pub fn new(
        input_dir: &Path,
        input_summary: FastqConfigSummary,
        samples: Vec<FastqReads>,
    ) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            input_summary,
            dependencies: DepMetadata::default(),
            samples,
        }
    }

    pub fn to_yaml(&mut self, override_args: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
        self.get_dependency(override_args);
        let output_path = generate_config_output_path(DEFAULT_READ_CLEANING_CONFIG);
        let writer = std::fs::File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn get_dependency(&mut self, override_args: Option<&str>) {
        let dep = FastpMetadata::new(override_args).get();

        match dep {
            Some(metadata) => self.dependencies = metadata,
            None => {
                panic!("Fastp dependency not found. Please, install fastp first");
            }
        }
    }
}
