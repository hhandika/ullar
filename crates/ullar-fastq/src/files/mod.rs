use std::path::Path;

use crate::types::reads::ReadAssignment;

use crate::types::reads::{FastqReads, SampleNameFormat};

pub mod finder;
pub mod reader;
pub mod reads;
pub mod regex;

pub fn find_and_assign_reads(
    input_dir: &Path,
    sample_name_format: &SampleNameFormat,
    recursive: bool,
) -> Vec<FastqReads> {
    let read_files = finder::FileFinder::new(input_dir)
        .find(recursive)
        .expect("Failed to find read files");
    ReadAssignment::new(&read_files, sample_name_format).assign()
}
