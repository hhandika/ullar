//! Initialize config file for assembly workflow.
//!
//! It allows ullar to generate a config file when previous workflow
//! is done using other tools.

use std::path::Path;

use crate::cli::args::InitArgs;
pub struct InitAssemblyConfig<'a> {
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
}

impl<'a> InitAssemblyConfig<'a> {
    pub fn new(args: &'a InitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            output_dir: &args.common.output,
        }
    }

    pub fn execute(&self) {
        unimplemented!()
    }
}
