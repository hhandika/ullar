pub mod metadata;
pub mod types;

use crate::bwa::types::BwaOutputFormat;
use std::path::Path;

pub struct BwaRunner<'a> {
    pub reference_path: &'a Path,
    pub query_read1: &'a Path,
    pub query_read2: Option<&'a Path>,
    pub output_path: &'a Path,
    pub output_format: BwaOutputFormat,
}

impl<'a> BwaRunner<'a> {
    pub fn new(
        reference_path: &'a Path,
        query_read1: &'a Path,
        query_read2: Option<&'a Path>,
        output_path: &'a Path,
        output_format: BwaOutputFormat,
    ) -> Self {
        Self {
            reference_path,
            query_read1,
            query_read2,
            output_path,
            output_format,
        }
    }
}
