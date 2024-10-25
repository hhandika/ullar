use std::path::{Path, PathBuf};

use segul::helper::{
    finder::IDs,
    types::{DataType, InputFmt},
};
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
        alignments: Vec<FileMetadata>,
        partition: Option<FileMetadata>,
    ) -> Self {
        Self {
            sample_counts,
            file_counts,
            concatenated: partition.is_some(),
            alignments,
            partition,
        }
    }

    pub fn from_sequence_files(
        sequences: &[PathBuf],
        format: &InputFmt,
        datatype: &DataType,
        partition: Option<&Path>,
    ) -> Self {
        let metadata = sequences
            .iter()
            .map(|f| {
                let mut meta = FileMetadata::new();
                meta.get(f);
                meta
            })
            .collect::<Vec<FileMetadata>>();
        let file_counts = metadata.len();
        let sample_counts = IDs::new(sequences, format, datatype).id_unique().len();
        let partition = partition.map(|p| {
            let mut meta = FileMetadata::new();
            meta.get(p);
            meta
        });
        Self {
            sample_counts,
            file_counts,
            concatenated: partition.is_some(),
            alignments: metadata,
            partition,
        }
    }

    /// Get raw alignment files from aligner
    pub fn get(alignments: Vec<FileMetadata>, sample_counts: usize) -> Self {
        let file_counts = alignments.len();
        Self::new(sample_counts, file_counts, alignments, None)
    }
}
