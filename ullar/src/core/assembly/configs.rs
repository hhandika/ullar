use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        clean::reports::CleanReadReport,
        deps::{spades::SpadesMetadata, DepMetadata},
    },
    helper::{configs::generate_config_output_path, fastq::FastqInput},
    types::reads::FastqReads,
};

pub const DEFAULT_ASSEMBLY_CONFIG: &str = "denovo_assembly";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyConfig {
    pub input_summary: FastqInput,
    pub dependencies: DepMetadata,
    pub samples: Vec<FastqReads>,
}

impl Default for AssemblyConfig {
    fn default() -> Self {
        Self {
            input_summary: FastqInput::default(),
            dependencies: DepMetadata::default(),
            samples: Vec::new(),
        }
    }
}

impl AssemblyConfig {
    pub fn new(input_summary: FastqInput, samples: Vec<FastqReads>) -> Self {
        Self {
            input_summary,
            dependencies: DepMetadata::default(),
            samples,
        }
    }

    /// Load the assembly config from a TOML file
    /// and return the AssemblyConfig instance
    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(config_path)?;
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
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_toml(
        &mut self,
        override_args: Option<&str>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.get_dependency(override_args);
        self.get_sample_counts();
        self.get_file_counts();
        let output_dir = generate_config_output_path(DEFAULT_ASSEMBLY_CONFIG);
        let toml = toml::to_string_pretty(self)?;
        fs::write(&output_dir, toml)?;
        Ok(output_dir)
    }

    #[deprecated(since = "0.4.0", note = "Use from_toml instead")]
    pub fn to_yaml(
        &mut self,
        override_args: Option<&str>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.get_dependency(override_args);
        let output_dir = generate_config_output_path(DEFAULT_ASSEMBLY_CONFIG);
        let writer = fs::File::create(&output_dir)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_dir)
    }

    pub fn from_clean_read_config(
        &mut self,
        reports: &[CleanReadReport],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.get_dependency(None);
        self.get_sample_counts();
        self.get_file_counts();
        self.samples = self.parse_fastp_report(reports);
        let output_path = generate_config_output_path(DEFAULT_ASSEMBLY_CONFIG);
        let toml = toml::to_string_pretty(self)?;
        fs::write(&output_path, toml)?;
        Ok(output_path)
    }

    fn get_dependency(&mut self, override_args: Option<&str>) {
        let dep = SpadesMetadata::new(override_args).get();

        match dep {
            Some(metadata) => self.dependencies = metadata,
            None => {
                panic!("SPAdes not found. Please, install spades first");
            }
        }
    }

    fn parse_fastp_report(&self, reports: &[CleanReadReport]) -> Vec<FastqReads> {
        let (tx, rx) = channel();
        reports.par_iter().for_each_with(tx, |tx, report| {
            let mut fastq = FastqReads::new();
            let sample_name = report.sample_name.clone();
            let parent_path = report.fastp_data.output_dir.to_path_buf();
            let read1_path = parent_path.join(&report.fastp_data.read1_filename);
            let read2_path = report
                .fastp_data
                .read2_filename
                .as_ref()
                .map(|read2| parent_path.join(read2));
            fastq.match_define_reads(sample_name, &read1_path, read2_path.as_deref());
            tx.send(fastq).expect("Failed to send fastq reads")
        });

        rx.iter().collect()
    }

    fn get_sample_counts(&mut self) {
        self.input_summary.sample_counts = self.samples.len();
    }

    fn get_file_counts(&mut self) {
        let read1 = self.samples.iter().filter(|s| s.read_1.is_some()).count();
        let read2 = self.samples.iter().filter(|s| s.read_2.is_some()).count();
        self.input_summary.file_counts = read1 + read2;
    }
}
