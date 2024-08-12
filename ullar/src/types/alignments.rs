use serde::{Deserialize, Serialize};

use crate::helper::files::FileMetadata;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlignmentFiles {
    pub sample_counts: usize,
    pub concatenated: bool,
    pub alignments: Vec<FileMetadata>,
    pub partition: Option<FileMetadata>,
}

impl AlignmentFiles {
    pub fn new(
        sample_counts: usize,
        concatenated: bool,
        alignments: Vec<FileMetadata>,
        partition: Option<FileMetadata>,
    ) -> Self {
        Self {
            sample_counts,
            concatenated,
            alignments,
            partition,
        }
    }
}
