use std::path::PathBuf;

use super::lastz::LastzOutput;
use crate::types::lastz::LastzOutputFormat;

pub struct LastzReport {
    pub output_dir: PathBuf,
    pub output_format: LastzOutputFormat,
    pub output_data: Vec<LastzOutput>,
}

impl LastzReport {
    pub fn new(
        output_dir: PathBuf,
        output_format: &LastzOutputFormat,
        output_data: Vec<LastzOutput>,
    ) -> Self {
        Self {
            output_dir,
            output_format: output_format.clone(),
            output_data,
        }
    }
}
