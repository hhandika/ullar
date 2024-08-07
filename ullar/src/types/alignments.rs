use serde::{Deserialize, Serialize};

use crate::helper::files::FileMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentFiles {
    pub sample_counts: usize,
    pub concatenated: bool,
    pub alignments: Vec<FileMetadata>,
}

impl Default for AlignmentFiles {
    fn default() -> Self {
        Self {
            sample_counts: 0,
            concatenated: false,
            alignments: Vec::new(),
        }
    }
}

impl AlignmentFiles {
    pub fn new(sample_counts: usize, concatenated: bool, alignments: Vec<FileMetadata>) -> Self {
        Self {
            sample_counts,
            concatenated,
            alignments,
        }
    }
}
