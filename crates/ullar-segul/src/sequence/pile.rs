//! Pile all sequence from the same sample to create a individual-level reference sequences

use std::path::PathBuf;

use segul::helper::types::InputFmt;

/// Create a pile sequence from multiple sequences of the same sample
/// Returns a FASTA formatted sequence
/// Each sequence is labeled with the file name it come from
pub struct PileSequence {
    pub input_files: Vec<PathBuf>,
    pub input_fmt: InputFmt,
}

impl PileSequence {
    /// Create a new PileSequence instance
    pub fn new(input_files: Vec<PathBuf>, input_fmt: InputFmt) -> Self {
        Self {
            input_files,
            input_fmt,
        }
    }
}
