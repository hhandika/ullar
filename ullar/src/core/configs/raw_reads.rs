use std::path::PathBuf;
use std::{error::Error, path::Path};

use serde::{Deserialize, Serialize};

use crate::types::reads::FastqReads;

use super::generate_config_output_path;

pub const DEFAULT_RAW_READ_CONFIG: &str = "raw_read";

pub enum FileMatchingStrategy {
    Regex,
    CharacterSplit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawReadConfig {
    pub input_dir: PathBuf,
    pub file_extension: String,
    pub sample_counts: usize,
    pub file_counts: usize,
    pub read_matching: ReadMatching,
    pub samples: Vec<FastqReads>,
}

impl Default for RawReadConfig {
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
            samples: Vec::new(),
        }
    }
}

impl RawReadConfig {
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
            samples,
        }
    }

    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_RAW_READ_CONFIG);
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
