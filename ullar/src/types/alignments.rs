use std::path::PathBuf;

use rayon::prelude::*;
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
    pub files: Vec<FileMetadata>,
}

impl AlignmentFiles {
    pub fn new(sample_counts: usize, file_counts: usize, files: Vec<FileMetadata>) -> Self {
        Self {
            sample_counts,
            file_counts,
            files,
        }
    }

    pub fn from_sequence_files(
        sequences: &[PathBuf],
        format: &InputFmt,
        datatype: &DataType,
    ) -> Self {
        let files = sequences
            .par_iter()
            .map(FileMetadata::from_path)
            .collect::<Vec<FileMetadata>>();
        let file_counts = files.len();
        let sample_counts = IDs::new(sequences, format, datatype).id_unique().len();
        Self {
            sample_counts,
            file_counts,
            files,
        }
    }

    /// Get raw alignment files from aligner
    pub fn get(alignments: Vec<FileMetadata>, sample_counts: usize) -> Self {
        let file_counts = alignments.len();
        Self::new(sample_counts, file_counts, alignments)
    }
}
