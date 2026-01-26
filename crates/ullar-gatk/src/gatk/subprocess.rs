//! GATK subprocess module

/// Structure to hold parameters for GATK Prepare step
/// Steps:
/// 1. Sort the input BAM/CRAM file
/// 2. Mark duplicates
/// 3. Add or replace read groups
///
pub struct GatkPrepare<'a> {
    pub input_path: &'a std::path::Path,
    pub output_path: &'a std::path::Path,
}

impl<'a> GatkPrepare<'a> {
    pub fn new(input_path: &'a std::path::Path, output_path: &'a std::path::Path) -> Self {
        Self {
            input_path,
            output_path,
        }
    }
}
