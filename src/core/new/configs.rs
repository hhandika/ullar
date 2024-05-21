use std::fs;
use std::path::PathBuf;
use std::{error::Error, path::Path};

use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG_FILE: &str = "raw_read.yaml";

pub enum FileMatchingStrategy {
    Regex,
    CharacterSplit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawReadConfig<'a> {
    input_dir: &'a Path,
    file_extension: &'a str,
    sample_counts: usize,
    file_counts: usize,
    read_matching: ReadMatching,
    data: &'a str,
}

impl<'a> RawReadConfig<'a> {
    pub fn new(
        input_dir: &'a Path,
        file_extension: &'a str,
        sample_counts: usize,
        file_counts: usize,
        read_matching: ReadMatching,
        data: &'a str,
    ) -> Self {
        Self {
            input_dir,
            sample_counts,
            file_counts,
            file_extension,
            read_matching,
            data,
        }
    }

    pub fn write_yaml(&self, output_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        fs::create_dir_all(output_dir)?;
        let output = output_dir.join(DEFAULT_CONFIG_FILE);
        let writer = std::fs::File::create(&output)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output)
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
