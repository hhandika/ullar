use std::path::Path;

use crate::cli::commands::alignment::AlignmentInitArgs;

pub struct AlignmentInit<'a> {
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
}

impl<'a> AlignmentInit<'a> {
    pub fn new(args: &'a AlignmentInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            output_dir: &args.common.output,
        }
    }

    pub fn init(&self) {
        unimplemented!()
    }
}
