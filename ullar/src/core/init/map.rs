//! Initialize config file for mapping contigs to reference sequence.
//!
//! Include support for phyluce for UCE analysis workflow.
use std::path::Path;

use crate::{
    cli::commands::init::MapInitArgs,
    core::{assembly::DEFAULT_ASSEMBLY_OUTPUT_DIR, configs::raw_reads::DEFAULT_CONFIG_DIR},
};

#[cfg(target_family = "unix")]
use crate::core::utils::symlinks::Symlinks;

pub struct InitMappingConfig<'a> {
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
    #[cfg(target_family = "unix")]
    pub phyluce: bool,
}

impl Default for InitMappingConfig<'_> {
    fn default() -> Self {
        Self {
            input_dir: Path::new(DEFAULT_ASSEMBLY_OUTPUT_DIR).as_ref(),
            output_dir: Path::new(DEFAULT_CONFIG_DIR).as_ref(),
            #[cfg(target_family = "unix")]
            phyluce: false,
        }
    }
}

impl<'a> InitMappingConfig<'a> {
    pub fn new(args: &'a MapInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            output_dir: &args.common.output,
            #[cfg(target_family = "unix")]
            phyluce: args.phyluce,
        }
    }

    pub fn initialize(&self) {
        if cfg!(target_family = "unix") {
            #[cfg(target_family = "unix")]
            self.execute_unix();
        } else {
            unimplemented!("Mapping contigs to reference sequence");
        }
    }

    #[cfg(target_family = "unix")]
    fn execute_unix(&self) {
        if self.phyluce {
            self.generate_phyluce_symlinks();
        } else {
            unimplemented!("Mapping contigs to reference sequence");
        }
    }

    #[cfg(target_family = "unix")]
    fn generate_phyluce_symlinks(&self) {
        let mut symlink = Symlinks::default();
        symlink.dir = self.input_dir;
        symlink.output_dir = self.output_dir;
        symlink.create();
    }
}
