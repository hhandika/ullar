use std::{error::Error, fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{core::utils::deps::DepMetadata, helper::files::FileMetadata, types::Task};

use super::generate_config_output_path;

pub const DEFAULT_LOCUS_CONFIG: &str = "mapped_contig";

#[derive(Debug, Serialize, Deserialize)]
pub struct MappedContigConfig {
    pub sample_counts: usize,
    pub file_counts: usize,
    pub dependencies: Vec<DepMetadata>,
    pub task: Task,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    pub contigs: Vec<FileMetadata>,
}

impl Default for MappedContigConfig {
    fn default() -> Self {
        Self {
            sample_counts: 0,
            file_counts: 0,
            dependencies: Vec::new(),
            task: Task::ContigMapping,
            override_args: None,
            contigs: Vec::new(),
        }
    }
}

impl MappedContigConfig {
    pub fn new(
        sample_counts: usize,
        file_counts: usize,
        dependencies: Vec<DepMetadata>,
        override_args: Option<String>,
        contigs: Vec<FileMetadata>,
    ) -> Self {
        Self {
            sample_counts,
            file_counts,
            dependencies,
            task: Task::ContigMapping,
            override_args,
            contigs,
        }
    }

    /// Get raw loci files
    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_LOCUS_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }
}
