use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use segul::helper::{
    finder::{IDs, SeqFileFinder},
    types::{DataType, InputFmt},
};
use serde::{Deserialize, Serialize};

use crate::{
    core::utils::deps::DepMetadata,
    helper::{
        configs::{generate_config_output_path, PreviousStep},
        files::FileMetadata,
    },
    types::Task,
};

pub const DEFAULT_LOCUS_CONFIG: &str = "mapped_contig";

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentConfig {
    pub sample_counts: usize,
    pub file_counts: usize,
    pub previous_step: PreviousStep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    pub contigs: Vec<FileMetadata>,
}

impl Default for AlignmentConfig {
    fn default() -> Self {
        Self {
            sample_counts: 0,
            file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contigs: Vec::new(),
        }
    }
}

impl AlignmentConfig {
    pub fn new(
        sample_counts: usize,
        file_counts: usize,
        task: Task,
        dependencies: Vec<DepMetadata>,
        override_args: Option<String>,
        contigs: Vec<FileMetadata>,
    ) -> Self {
        Self {
            sample_counts,
            file_counts,
            previous_step: PreviousStep::with_dependencies(task, dependencies),
            override_args,
            contigs,
        }
    }

    pub fn init(&mut self, input_dir: &Path, previous_step: Option<PreviousStep>) {
        let sequence_files = self.find_files(input_dir);
        match previous_step {
            Some(step) => self.previous_step = step,
            None => self.previous_step = PreviousStep::new(Task::Unknown),
        }
        self.file_counts = sequence_files.len();
        self.sample_counts = self.count_samples(&sequence_files);
        self.contigs = self.get_metadata(&sequence_files);
    }

    /// Get raw loci files
    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_LOCUS_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn find_files(&self, input_dir: &Path) -> Vec<PathBuf> {
        let input_format = InputFmt::Fasta;
        let sequence_files = SeqFileFinder::new(input_dir).find_recursive_only(&input_format);
        sequence_files
    }

    fn count_samples(&self, sequence_files: &[PathBuf]) -> usize {
        let format = InputFmt::Fasta;
        let datatype = DataType::Dna;
        let unique_ids = IDs::new(sequence_files, &format, &datatype).id_unique();
        unique_ids.len()
    }

    fn get_metadata(&self, sequence_files: &[PathBuf]) -> Vec<FileMetadata> {
        sequence_files
            .iter()
            .map(|f| {
                let mut file = FileMetadata::new();
                file.get(f);
                file
            })
            .collect()
    }
}
