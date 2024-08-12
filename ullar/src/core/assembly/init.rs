//! Initialize config file for assembly workflow.
//!
//! It allows ullar to generate a config file when previous workflow
//! is done using other tools.

use std::path::Path;

use crate::cli::commands::assembly::AssemblyInitArgs;
pub struct AssemblyInit<'a> {
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
}

impl<'a> AssemblyInit<'a> {
    pub fn new(args: &'a AssemblyInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            output_dir: &args.common.output,
        }
    }

    pub fn init(&self) {
        unimplemented!()
    }
}
