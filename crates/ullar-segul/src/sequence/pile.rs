//! Pile all sequence from the same sample to create a individual-level reference sequences

use std::path::PathBuf;

use segul::helper::types::InputFmt;

/// Create a pile sequence from multiple sequences of the same sample
/// Returns a FASTA formatted sequence
/// Each sequence is labeled with the file name it come from
pub struct PileSequence {
    pub input_files: Vec<PathBuf>,
    pub input_fmt: InputFmt,
    pub output_dir: PathBuf,
}

impl PileSequence {
    /// Create a new PileSequence instance
    pub fn new(input_files: Vec<PathBuf>, input_fmt: InputFmt) -> Self {
        Self {
            input_files,
            input_fmt,
            output_dir: PathBuf::new(),
        }
    }

    pub fn output_dir<P: AsRef<std::path::Path>>(&mut self, output_dir: P) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        self
    }

    /// Run the pile sequence process
    pub fn pile(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Pile sequences from files: {:?}", self.input_files);
        Ok(())
    }
}
