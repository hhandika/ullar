pub mod init;
pub mod mafft;

use std::{error::Error, fs, path::Path};

use colored::Colorize;

use crate::{cli::commands::alignment::AlignmentArgs, types::Task};

use super::configs::alignment::SequenceAlignments;

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
    pub override_args: Option<&'a str>,
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
        override_args: Option<&'a str>,
        skip_config_check: bool,
    ) -> Self {
        Self {
            config_path,
            ignore_checksum,
            output_dir,
            override_args,
            skip_config_check,
            task: Task::AligningSequences,
        }
    }
    /// Initialize a new Alignment instance
    /// from the command line arguments
    pub fn from_arg(args: &'a AlignmentArgs) -> Self {
        Self {
            config_path: &args.config,
            ignore_checksum: args.common.ignore_checksum,
            output_dir: &args.output,
            override_args: args.common.override_args.as_deref(),
            skip_config_check: args.common.skip_config_check,
            task: Task::AligningSequences,
        }
    }

    /// Align the sequences based on the configuration
    ///
    /// Steps:
    /// 1. Parse the configuration file
    /// 2. Log the input summary
    /// 3.
    pub fn align(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
    }

    fn parse_config(&self) -> Result<SequenceAlignments, Box<dyn Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: SequenceAlignments = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    fn log_input(&self, config: &SequenceAlignments) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config path", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.sample_counts);
        log::info!("{:18}: {}", "File counts", config.file_counts);
        log::info!("{:18}: {}", "Task", self.task);
    }
}
