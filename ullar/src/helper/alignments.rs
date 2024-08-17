use std::path::PathBuf;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use segul::helper::{
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
    pub fn new() -> Self {
        Self {
            summary: CandidateAlignmentSummary::new(),
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
        self.summary.count(total_found, self.final_files.len());
    }

    fn is_single_sequence(&self, contig: &PathBuf) -> bool {
        let datatype = DataType::Dna;
        let input_fmt = InputFmt::Auto;
        let (_, header) = SeqParser::new(contig, &datatype).parse(&input_fmt);
        if header.ntax < 2 {
            return true;
        }
        false
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct CandidateAlignmentSummary {
    pub total_found: usize,
    pub skipped: usize,
    pub final_count: usize,
}

impl CandidateAlignmentSummary {
    pub fn new() -> Self {
        Self {
            total_found: 0,
            skipped: 0,
            final_count: 0,
        }
    }

    pub fn count(&mut self, found: usize, final_files: usize) {
        self.total_found = found;
        self.skipped = found - final_files;
        self.final_count = final_files;
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use segul::helper::finder::SeqFileFinder;

    use super::*;

    #[test]
    fn test_candidate_alignment_summary() {
        let mut summary = CandidateAlignmentSummary::new();
        summary.count(10, 2);
        assert_eq!(summary.total_found, 10);
        assert_eq!(summary.skipped, 8);
        assert_eq!(summary.final_count, 2);
    }

    #[test]
    fn test_filtered_contigs() {
        let path = Path::new("tests/data/alignments");
        let files = SeqFileFinder::new(&path).find(&InputFmt::Auto);
        let mut filter = FilteredSequenceFiles::new();
        filter.filter_single_sequence(&files);
        assert_eq!(filter.summary.total_found, 4);
        assert_eq!(filter.summary.skipped, 1);
        assert_eq!(filter.summary.final_count, 3);
    }
}
