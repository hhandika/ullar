pub mod configs;
pub mod init;
pub mod mafft;
pub mod reports;

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
};

use colored::Colorize;
use configs::AlignmentConfig;
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

use super::deps::mafft::MafftMetadata;

pub const DEFAULT_ALIGNMENT_OUTPUT_DIR: &str = "alignments";

pub struct SequenceAlignment<'a> {
    /// Path to the alignment config file
    pub config_path: &'a Path,
    /// Output directory to store the alignments
    pub output_dir: &'a Path,
    /// Runner options for the alignment
    pub runner: RunnerOptions<'a>,
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
    pub fn align(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Parsing config file");
        let config = self.parse_config().expect("Failed to parse config");
        spinner.finish_with_message(format!("{} Finished parsing config file\n", "✔".green()));
        self.log_input(&config);
        PathCheck::new(self.output_dir, true, self.runner.force).prompt_exists(self.runner.dry_run);
        let reports = self.par_align(&config.contigs);
        self.log_final_output(&reports);
    }

    fn parse_config(&self) -> Result<AlignmentConfig, Box<dyn Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: AlignmentConfig = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    fn par_align(&self, input_files: &[FileMetadata]) -> MafftReport {
        let progress_bar = common::init_progress_bar(input_files.len() as u64);
        log::info!("{}", "Aligning sequences".cyan());
        progress_bar.set_message("Alignments");
        let (tx, rx) = mpsc::channel();
        input_files.par_iter().for_each_with(tx, |tx, file| {
            let output = self.align_mafft(file);
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

    fn align_mafft(&self, input: &FileMetadata) -> Result<PathBuf, Box<dyn Error>> {
        let mafft = MafftRunner::new(input, self.output_dir, self.runner.override_args);
        mafft.run()
    }

    fn log_input(&self, config: &AlignmentConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config path", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.sample_counts);
        log::info!("{:18}: {}", "File found", config.file_summary.total_found);
        log::info!("{:18}: {}", "File skipped", config.file_summary.skipped);
        log::info!(
            "{:18}: {}",
            "Final file count",
            config.file_summary.final_count
        );
        log::info!("{:18}: {}", "Task", self.task);
        self.log_mafft_info();
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

    fn log_mafft_info(&self) {
        let dep = MafftMetadata::new(None).get();
        match dep {
            Some(mafft) => log::info!("{:18}: {} v{}", "Aligner", "MAFFT", mafft.version),
            None => log::info!("{:18}: {}", "Aligner", "MAFFT"),
        }

        let params = match self.runner.override_args {
            Some(args) => args,
            None => DEFAULT_MAFFT_PARAMS,
        };
        log::info!("{:18}: {}\n", "Parameters", params);
    }
}
