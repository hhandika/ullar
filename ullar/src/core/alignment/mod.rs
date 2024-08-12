pub mod init;
pub mod mafft;

use std::path::Path;

use crate::{cli::commands::alignment::AlignmentArgs, types::Task};

pub const DEFAULT_ALIGNMENT_OUTPUT_DIR: &str = "alignments";

pub struct Alignment<'a> {
    /// Path to the alignment configuration file
    pub config_path: &'a Path,
    /// Should the SHA256 checksum be checked
    /// before aligning the files
    pub ignore_checksum: bool,
    /// Output directory to store the alignments
    pub output_dir: &'a Path,
    /// Optional parameters for the alignment process
    pub optional_params: Option<&'a str>,
    /// Check config for errors
    pub skip_config_check: bool,
    #[allow(dead_code)]
    task: Task,
}

impl<'a> Alignment<'a> {
    /// Initialize a new Alignment instance
    /// with the given parameters
    pub fn new(
        config_path: &'a Path,
        ignore_checksum: bool,
        output_dir: &'a Path,
        optional_params: Option<&'a str>,
        skip_config_check: bool,
    ) -> Self {
        Self {
            config_path,
            ignore_checksum,
            output_dir,
            optional_params,
            skip_config_check,
            task: Task::Alignment,
        }
    }
    /// Initialize a new Alignment instance
    /// from the command line arguments
    pub fn from_arg(args: &'a AlignmentArgs) -> Self {
        Self {
            config_path: &args.config,
            ignore_checksum: args.common.ignore_checksum,
            output_dir: &args.output,
            optional_params: args.common.override_args.as_deref(),
            skip_config_check: args.common.skip_config_check,
            task: Task::Alignment,
        }
    }

    pub fn align(&self) {
        // Align the sequences
    }
}
