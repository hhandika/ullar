use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    core::utils::deps::DepMetadata,
    helper::{
        configs::{generate_config_output_path, PreviousStep},
        files::FileMetadata,
    },
    types::Task,
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
    pub reference_file: FileMetadata,
}

impl Default for MappedContigConfig {
    fn default() -> Self {
        Self {
            contig_file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contig_files: Vec::new(),
            reference_file: FileMetadata::new(),
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
            reference_file: FileMetadata::new(),
        }
    }

    pub fn init(
        &mut self,
        contig_dir: &Path,
        reference_dir: &Path,
        previous_step: Option<PreviousStep>,
    ) {
        let sequence_files = self.find_contig_files(contig_dir);
        match previous_step {
            Some(step) => self.previous_step = step,
            None => self.previous_step = PreviousStep::new(Task::Unknown),
        }
        self.contig_files = self.get_metadata(&sequence_files);
        self.reference_file.get(reference_dir);
    }

    /// Get raw loci files
    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_LOCUS_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn find_contig_files(&self, input_dir: &Path) -> Vec<PathBuf> {
        WalkDir::new(input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(|e| e.is_file())
            .filter(|e| re_matches_contigs(e))
            .collect::<Vec<PathBuf>>()
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

fn re_matches_contigs(path: &Path) -> bool {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(CONTIG_REGEX).expect("Failed to compile regex"));
    let file_name = path.file_name().expect("Failed to get file name");
    RE.is_match(
        file_name
            .to_str()
            .expect("Failed to convert file name to string"),
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_mapped_contig_config() {
        let config = MappedContigConfig::default();
        let contig_dir = Path::new("tests/contigs");
        config.find_contig_files(contig_dir);
        assert_eq!(config.contig_files.len(), 1);
    }
}
