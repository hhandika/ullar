use std::path::PathBuf;
use std::{error::Error, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::deps::DepMetadata;
use crate::helper::configs::generate_config_output_path;
use crate::types::reads::FastqReads;

pub const DEFAULT_READ_CLEANING_CONFIG: &str = "read_cleaning";

pub enum FileMatchingStrategy {
    Regex,
    CharacterSplit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanReadConfig {
    pub input_dir: PathBuf,
    pub file_extension: String,
    pub sample_counts: usize,
    pub file_counts: usize,
    pub read_matching: ReadMatching,
    pub dependencies: DepMetadata,
    pub samples: Vec<FastqReads>,
}

impl Default for CleanReadConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            file_extension: String::new(),
            sample_counts: 0,
            file_counts: 0,
            read_matching: ReadMatching {
                regex: None,
                character_split: None,
            },
            dependencies: DepMetadata::default(),
            samples: Vec::new(),
        }
    }
}

impl CleanReadConfig {
    pub fn new(
        input_dir: &Path,
        file_extension: String,
        sample_counts: usize,
        file_counts: usize,
        read_matching: ReadMatching,
        samples: Vec<FastqReads>,
    ) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            sample_counts,
            file_counts,
            file_extension,
            read_matching,
            dependencies: DepMetadata::default(),
            samples,
        }
    }

    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_READ_CLEANING_CONFIG);
        let writer = std::fs::File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadMatching {
    #[serde(skip_serializing_if = "Option::is_none")]
    regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    character_split: Option<ReadMatchingCharacterSplit>,
}

impl ReadMatching {
    pub fn regex(regex: String) -> Self {
        Self {
            regex: Some(regex),
            character_split: None,
        }
    }

    pub fn character_split(separator: char, word_counts: usize) -> Self {
        Self {
            regex: None,
            character_split: Some(ReadMatchingCharacterSplit {
                separator,
                word_counts,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadMatchingCharacterSplit {
    separator: char,
    word_counts: usize,
}
