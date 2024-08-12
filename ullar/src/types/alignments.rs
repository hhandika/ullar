use serde::{Deserialize, Serialize};

use crate::helper::files::FileMetadata;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlignmentFiles {
    pub sample_counts: usize,
    pub file_counts: usize,
    pub concatenated: bool,
    pub alignments: Vec<FileMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<FileMetadata>,
}

impl AlignmentFiles {
    pub fn new(
        sample_counts: usize,
        file_counts: usize,
        concatenated: bool,
        alignments: Vec<FileMetadata>,
        partition: Option<FileMetadata>,
    ) -> Self {
        Self {
            sample_counts,
            file_counts,
            concatenated,
            alignments,
            partition,
        }
    }

    /// Get raw alignment files from aligner
    pub fn get_raw(&mut self, alignments: Vec<FileMetadata>, sample_counts: usize) {
        self.alignments = alignments;
        self.sample_counts = sample_counts;
        self.file_counts = self.alignments.len();
    }
}
