use std::fs;
use std::path::PathBuf;
use std::{error::Error, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::deps::fastp::FastpMetadata;
use crate::deps::DepMetadata;
use crate::helper::common::UllarConfig;
use crate::helper::configs::generate_config_output_path;
use crate::helper::fastq::FastqInput;
use crate::types::reads::FastqReads;

pub const DEFAULT_READ_CLEANING_CONFIG: &str = "read_cleaning";

pub enum FileMatchingStrategy {
    Regex,
    CharacterSplit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanReadConfig {
    #[serde(flatten)]
    pub app: UllarConfig,
    pub input: FastqInput,
    pub dependencies: DepMetadata,
    pub samples: Vec<FastqReads>,
}

impl Default for CleanReadConfig {
    fn default() -> Self {
        Self {
            app: UllarConfig::default(),
            input: FastqInput::default(),
            dependencies: DepMetadata::default(),
            samples: Vec::new(),
        }
    }
}

impl CleanReadConfig {
    pub fn new(input: FastqInput, samples: Vec<FastqReads>) -> Self {
        Self {
            app: UllarConfig::default(),
            input,
            dependencies: DepMetadata::default(),
            samples,
        }
    }

    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Input config path: {}", config_path.display()))?;
        let ext = config_path.extension().unwrap_or_default();
        if ext == "yaml" || ext == "yml" {
            let config = serde_yaml::from_str(&content)?;
            let toml = toml::to_string_pretty(&config)?;
            let config_path = config_path.with_extension("toml");
            fs::write(&config_path, toml)?;
            log::info!(
                "Converted YAML config to TOML format: {}",
                config_path.display()
            );
            return Ok(config);
        }
        let config: CleanReadConfig = toml::from_str(&content)?;

        Ok(config)
    }

    #[deprecated(since = "0.5.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&mut self, override_args: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
        self.get_dependency(override_args);
        let output_path = generate_config_output_path(DEFAULT_READ_CLEANING_CONFIG);
        let writer = std::fs::File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    pub fn to_toml(&mut self, override_args: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
        self.get_dependency(override_args);
        let output_path = generate_config_output_path(DEFAULT_READ_CLEANING_CONFIG);
        let toml = toml::to_string_pretty(&self)?;
        std::fs::write(&output_path, toml)?;
        Ok(output_path)
    }

    fn get_dependency(&mut self, override_args: Option<&str>) {
        let dep = FastpMetadata::new(override_args).get();

        match dep {
            Some(metadata) => self.dependencies = metadata,
            None => {
                panic!("Fastp dependency not found. Please, install fastp");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_read_config() {
        let input_dir = Path::new("tests/data/configs/clean_read.toml");
        let config = CleanReadConfig::from_toml(input_dir).unwrap();
        let input = PathBuf::from("datasets/rawreads/");
        let sample_name = "Bunomys_chrysocomus_ABCD1234";

        assert_eq!(config.input.input_dir, input);
        assert_eq!(config.input.sample_counts, 1);
        assert_eq!(config.samples.len(), 1);
        assert_eq!(config.dependencies.name, "fastp");
        assert_eq!(config.dependencies.version, "0.23.4");
        assert_eq!(config.samples[0].sample_name, sample_name);
        assert_eq!(
            config.samples[0].read_1.as_ref().unwrap().file_name,
            "Bunomys_chrysocomus_ABCD1234_READ1.fq.gz"
        );
    }
}
