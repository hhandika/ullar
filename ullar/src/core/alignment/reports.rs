use std::path::PathBuf;

use segul::helper::{
    finder::IDs,
    types::{DataType, InputFmt},
};

use crate::{helper::files::FileMetadata, types::alignments::AlignmentFiles};

pub struct MafftReport {
    /// Alignment data
    pub alignments: AlignmentFiles,
    /// Alignment format
    /// Use as input to parse the alignment files
    ///     using segul
    pub format: InputFmt,
    /// Datatype
    pub datatype: DataType,
}

impl Default for MafftReport {
    fn default() -> Self {
        Self::new()
    }
}

impl MafftReport {
    /// Initialize a new MafftReport instance
    pub fn new() -> Self {
        Self {
            alignments: AlignmentFiles::default(),
            format: InputFmt::Fasta,
            datatype: DataType::Dna,
        }
    }

    pub fn create(&mut self, mafft_outputs: &[PathBuf]) {
        assert!(!mafft_outputs.is_empty(), "No alignment files found");
        let metadata = self.get_metadata(mafft_outputs);
        let sample_counts = self.get_sample_count(&metadata);
        self.alignments = AlignmentFiles::get(metadata, sample_counts);
    }

    fn get_metadata(&self, files: &[PathBuf]) -> Vec<FileMetadata> {
        let mut metadata: Vec<FileMetadata> = Vec::new();
        files.iter().for_each(|f| {
            metadata.push(FileMetadata::from_path(f));
        });

        metadata
    }

    fn get_sample_count(&self, files: &[FileMetadata]) -> usize {
        let file_paths: Vec<PathBuf> = files
            .iter()
            .map(|f| f.parent_dir.join(&f.file_name))
            .collect();
        let unique_ids = IDs::new(&file_paths, &self.format, &self.datatype).id_unique();
        unique_ids.len()
    }
}
