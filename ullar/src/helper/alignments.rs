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
pub struct FilteredContigs {
    pub summary: CandidateAlignmentSummary,
    pub final_contigs: Vec<PathBuf>,
}

impl FilteredContigs {
    pub fn new() -> Self {
        Self {
            summary: CandidateAlignmentSummary::new(),
            final_contigs: Vec::new(),
        }
    }

    pub fn filter_single_sequence(&mut self, contigs: &[PathBuf]) {
        let total_found = contigs.len();
        self.final_contigs = contigs
            .par_iter()
            .filter(|contig| !self.is_single_sequence(contig))
            .map(|contig| contig.to_path_buf())
            .collect();
        self.summary.count(total_found, self.final_contigs.len());
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

    pub fn count(&mut self, found: usize, skipped: usize) {
        self.total_found = found;
        self.skipped = skipped;
        self.final_count = self.total_found - self.skipped;
    }
}
