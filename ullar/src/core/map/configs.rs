use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    core::utils::deps::DepMetadata,
    helper::{
        configs::{generate_config_output_path, PreviousStep},
        files::{FileFinder, FileMetadata},
    },
    types::{SupportedFormats, Task},
};

pub const DEFAULT_LOCUS_CONFIG: &str = "mapped_contig";

pub const CONTIG_REGEX: &str = r"(?i)(contig*)";

#[derive(Debug, Serialize, Deserialize)]
pub struct MappedContigConfig {
    /// Total number of contig files
    pub contig_file_counts: usize,
    pub previous_step: PreviousStep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    pub contig_files: Vec<FileMetadata>,
}

impl Default for MappedContigConfig {
    fn default() -> Self {
        Self {
            contig_file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contig_files: Vec::new(),
        }
    }
}

impl MappedContigConfig {
    pub fn new(
        file_counts: usize,
        task: Task,
        dependencies: Vec<DepMetadata>,
        override_args: Option<String>,
    ) -> Self {
        Self {
            contig_file_counts: file_counts,
            previous_step: PreviousStep::with_dependencies(task, dependencies),
            override_args,
            contig_files: Vec::new(),
        }
    }

    pub fn from_contig_dir(&mut self, contig_dir: &Path, previous_step: Option<PreviousStep>) {
        let sequence_files = self.find_contig_files(contig_dir);
        if sequence_files.is_empty() {
            log::error!(
                "No contig files found in directory: {}",
                contig_dir.display()
            );
            return;
        }
        self.assign_values(&sequence_files, previous_step);
    }

    pub fn from_contig_paths(&mut self, contigs: &[PathBuf], previous_step: Option<PreviousStep>) {
        if contigs.is_empty() {
            log::warn!("No contig files found in input");
            return;
        }
        self.assign_values(contigs, previous_step);
    }

    fn assign_values(&mut self, contigs: &[PathBuf], previous_step: Option<PreviousStep>) {
        self.contig_file_counts = contigs.len();
        match previous_step {
            Some(step) => self.previous_step = step,
            None => self.previous_step = PreviousStep::new(Task::Unknown),
        }
        self.contig_files = self.get_metadata(contigs);
    }

    /// Get raw loci files
    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_LOCUS_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn find_contig_files(&self, input_dir: &Path) -> Vec<PathBuf> {
        let format = SupportedFormats::Contigs;
        FileFinder::new(input_dir, &format)
            .find(true)
            .expect("Failed to find contig files")
    }

    fn get_metadata(&self, sequence_files: &[PathBuf]) -> Vec<FileMetadata> {
        assert!(
            !sequence_files.is_empty(),
            "No sequence files found in the input directory"
        );
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
