pub mod configs;
pub mod init;
pub mod mafft;
pub mod reports;

use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::mpsc,
};

use colored::Colorize;
use configs::{AlignmentConfig, ALIGNER_DEPENDENCY};
use mafft::{MafftRunner, DEFAULT_MAFFT_PARAMS};
use rayon::prelude::*;
use reports::MafftReport;

use crate::{
    cli::commands::alignment::AlignmentArgs,
    helper::{
        common,
        files::{FileMetadata, PathCheck},
    },
    types::{runner::RunnerOptions, Task},
};

use crate::core::deps::mafft::MafftMetadata;

use super::deps::DepMetadata;

pub const DEFAULT_ALIGNMENT_OUTPUT_DIR: &str = "alignments";

pub struct SequenceAlignment<'a> {
    /// Path to the alignment config file
    pub config_path: &'a Path,
    /// Output directory to store the alignments
    pub output_dir: &'a Path,
    /// Runner options for the alignment
    pub runner: RunnerOptions,
    task: Task,
}

impl<'a> SequenceAlignment<'a> {
    /// Initialize a new Alignment instance
    /// with the given parameters
    pub fn new(config_path: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            output_dir,
            runner: RunnerOptions::default(),
            task: Task::SequenceAlignment,
        }
    }
    /// Initialize a new Alignment instance
    /// from the command line arguments
    pub fn from_arg(args: &'a AlignmentArgs) -> Self {
        Self {
            config_path: &args.config,
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::SequenceAlignment,
        }
    }

    /// Align the sequences based on the configuration
    ///
    /// Steps:
    /// 1. Parse the config file
    /// 2. Log the input summary
    /// 3. Check if the output directory exists
    /// 4. Check configuration
    /// 6. If dry-run, print the summary and exit
    /// 7. Align the sequences
    pub fn align(&mut self) {
        let spinner = common::init_spinner();
        spinner.set_message("Parsing config file");
        let config = self.parse_config().expect("Failed to parse config");
        spinner.finish_with_message(format!("{} Finished parsing config file\n", "✔".green()));
        let aligner = config.dependencies.get(ALIGNER_DEPENDENCY);
        let updated_dep = MafftMetadata::new().update(aligner);
        self.log_input(&config, &updated_dep);
        PathCheck::new(self.output_dir)
            .is_dir()
            .with_force_overwrite(self.runner.overwrite)
            .prompt_exists(self.runner.dry_run);
        let reports = self.par_align(&config.sequences, &updated_dep);
        self.log_final_output(&reports);
    }

    fn parse_config(&self) -> Result<AlignmentConfig, Box<dyn Error>> {
        let config = AlignmentConfig::from_toml(self.config_path)?;
        Ok(config)
    }

    fn par_align(&mut self, sequences: &[FileMetadata], mafft: &DepMetadata) -> MafftReport {
        let progress_bar = common::init_progress_bar(sequences.len() as u64);
        log::info!("{}", "Aligning sequences".cyan());
        progress_bar.set_message("Alignments");
        let (tx, rx) = mpsc::channel();
        sequences.par_iter().for_each_with(tx, |tx, file| {
            let output = self.align_mafft(file, mafft);
            match output {
                Ok(path) => tx.send(path).expect("Failed to send output path"),
                Err(e) => log::error!("Failed to align {}: {}", file.file_name.red(), e),
            }
            progress_bar.inc(1);
        });

        let output_paths: Vec<PathBuf> = rx.iter().collect();
        progress_bar.finish_with_message(format!("{} Finished alignments\n", "✔".green()));
        let mut report = MafftReport::new();
        report.create(&output_paths);

        report
    }

    fn align_mafft(
        &self,
        input: &FileMetadata,
        mafft: &DepMetadata,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let mafft = MafftRunner::new(input, self.output_dir, mafft);
        mafft.run()
    }

    fn log_input(&self, config: &AlignmentConfig, dep: &DepMetadata) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config path", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.input.sample_counts);
        log::info!("{:18}: {}", "File found", config.input.total_files);
        log::info!("{:18}: {}", "File skipped", config.input.file_skipped);
        log::info!("{:18}: {}", "Final file count", config.input.file_counts);
        log::info!("{:18}: {}", "Task", self.task);
        log::info!("{:18}: {} v{}", "Aligner", "MAFFT", dep.version);
        let params = dep
            .override_args
            .as_ref()
            .unwrap_or(&DEFAULT_MAFFT_PARAMS.to_string())
            .to_string();
        log::info!("{:18}: {}\n", "Parameters", params);
    }

    fn log_final_output(&self, reports: &MafftReport) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Directory", self.output_dir.display());
        log::info!("{:18}: {}", "File counts", reports.alignments.file_counts);
        log::info!(
            "{:18}: {}",
            "Sample counts",
            reports.alignments.sample_counts
        );
    }
}
