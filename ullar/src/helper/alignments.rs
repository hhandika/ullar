use std::path::{Path, PathBuf};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use segul::helper::{
    finder::IDs,
    sequence::SeqParser,
    types::{DataType, InputFmt},
};
use serde::{Deserialize, Serialize};

/// Data structure to filter
///     problematic contigs.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FilteredSequenceFiles {
    pub summary: CandidateAlignmentSummary,
    pub final_files: Vec<PathBuf>,
}

impl FilteredSequenceFiles {
    pub fn new(input_dir: &Path) -> Self {
        Self {
            summary: CandidateAlignmentSummary::new(input_dir),
            final_files: Vec::new(),
        }
    }

    pub fn filter_single_sequence(&mut self, contigs: &[PathBuf]) {
        let total_found = contigs.len();
        self.final_files = contigs
            .par_iter()
            .filter(|contig| !self.is_single_sequence(contig))
            .map(|contig| contig.to_path_buf())
            .collect();
        let sample_count = self.count_samples(&self.final_files);
        self.summary
            .count(total_found, self.final_files.len(), sample_count);
    }

    // We use SEGUL to count the number of samples in the sequence files
    // Input format is automatically detected
    // because during the sequence file finding process, we already know the input format
    // is sequence files.
    fn count_samples(&self, sequence_files: &[PathBuf]) -> usize {
        let format = InputFmt::Auto;
        let datatype = DataType::Dna;
        let unique_ids = IDs::new(sequence_files, &format, &datatype).id_unique();
        unique_ids.len()
    }

    fn is_single_sequence(&self, contig: &Path) -> bool {
        let datatype = DataType::Dna;
        let input_fmt = InputFmt::Auto;
        let (_, header) = SeqParser::new(contig, &datatype).parse(&input_fmt);
        if header.ntax < 2 {
            return true;
        }
        false
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CandidateAlignmentSummary {
    pub input_dir: PathBuf,
    pub sample_counts: usize,
    pub total_files: usize,
    pub file_skipped: usize,
    /// Final count of files
    /// skipping single sequence files
    pub file_counts: usize,
}

impl CandidateAlignmentSummary {
    pub fn new(input_dir: &Path) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            sample_counts: 0,
            total_files: 0,
            file_skipped: 0,
            file_counts: 0,
        }
    }

    pub fn count(&mut self, found: usize, final_files: usize, samples: usize) {
        self.sample_counts = samples;
        self.total_files = found;
        self.file_counts = final_files;
        self.file_skipped = found - final_files;
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use segul::helper::finder::SeqFileFinder;

    use super::*;

    #[test]
    fn test_candidate_alignment_summary() {
        let mut summary = CandidateAlignmentSummary::new(Path::new("tests/data/alignments"));
        summary.count(10, 2, 1);
        assert_eq!(summary.total_files, 10);
        assert_eq!(summary.file_skipped, 8);
        assert_eq!(summary.file_counts, 2);
        assert_eq!(summary.sample_counts, 1);
    }

    #[test]
    fn test_filtered_contigs() {
        let path = Path::new("tests/data/alignments");
        let files = SeqFileFinder::new(&path).find(&InputFmt::Auto);
        let mut filter = FilteredSequenceFiles::new(path);
        filter.filter_single_sequence(&files);
        assert_eq!(filter.summary.total_files, 4);
        assert_eq!(filter.summary.file_skipped, 1);
        assert_eq!(filter.final_files.len(), 3);
        assert_eq!(filter.summary.sample_counts, 3);
    }
}
