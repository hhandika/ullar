use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use segul::helper::{
    finder::{IDs, SeqFileFinder},
    types::{DataType, InputFmt},
};
use serde::{Deserialize, Serialize};

use crate::{
    core::deps::DepMetadata,
    helper::{
        alignments::{CandidateAlignmentSummary, FilteredSequenceFiles},
        configs::{generate_config_output_path, PreviousStep},
        files::FileMetadata,
    },
    types::Task,
};

pub const DEFAULT_ALIGNMENT_CONFIG: &str = "sequence_alignment";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlignmentConfig {
    pub file_summary: CandidateAlignmentSummary,
    pub sample_counts: usize,
    pub previous_step: PreviousStep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    pub sequences: Vec<FileMetadata>,
}

impl AlignmentConfig {
    pub fn new(
        task: Task,
        dependencies: Vec<DepMetadata>,
        override_args: Option<String>,
        sequences: Vec<FileMetadata>,
    ) -> Self {
        Self {
            file_summary: CandidateAlignmentSummary::default(),
            sample_counts: 0,
            previous_step: PreviousStep::with_dependencies(task, dependencies),
            override_args,
            sequences,
        }
    }

    pub fn init(&mut self, input_dir: &Path, previous_step: Option<PreviousStep>) {
        let sequence_files = self.find_files(input_dir);
        match previous_step {
            Some(step) => self.previous_step = step,
            None => self.previous_step = PreviousStep::new(Task::Unknown),
        }
        self.file_summary = sequence_files.summary;
        self.sample_counts = self.count_samples(&sequence_files.final_files);
        self.sequences = self.get_metadata(&sequence_files.final_files);
    }

    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn Error>> {
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

    pub fn to_toml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_ALIGNMENT_CONFIG);
        let config = toml::to_string_pretty(self)?;
        fs::write(&output_path, config)?;
        Ok(output_path)
    }

    /// Get raw loci files
    #[deprecated(since = "0.4.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_ALIGNMENT_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn find_files(&self, input_dir: &Path) -> FilteredSequenceFiles {
        let input_format = InputFmt::Fasta;
        let sequence_files = SeqFileFinder::new(input_dir).find_recursive_only(&input_format);
        self.filter_problematic_contigs(&sequence_files)
    }

    fn filter_problematic_contigs(&self, contigs: &[PathBuf]) -> FilteredSequenceFiles {
        let mut filtered_contigs = FilteredSequenceFiles::new();
        filtered_contigs.filter_single_sequence(contigs);
        filtered_contigs
    }

    fn count_samples(&self, sequence_files: &[PathBuf]) -> usize {
        let format = InputFmt::Auto;
        let datatype = DataType::Dna;
        let unique_ids = IDs::new(sequence_files, &format, &datatype).id_unique();
        unique_ids.len()
    }

    fn get_metadata(&self, sequence_files: &[PathBuf]) -> Vec<FileMetadata> {
        sequence_files
            .par_iter()
            .map(|f| {
                let mut file = FileMetadata::new();
                file.get(f);
                file
            })
            .collect()
    }
}
