//! Clean raw read files using Fastp
pub mod fastp;

use std::path::Path;
use std::sync::mpsc;

use colored::Colorize;
use rayon::prelude::*;

use crate::cli::args::CleanArgs;
use crate::helper::reads::{FastqReads, RawReadChecker};
use crate::helper::utils;

use super::new::configs::RawReadConfig;

pub struct ReadCleaner<'a> {
    /// Path to the raw read configuration file
    pub config_path: &'a Path,
    /// Should the SHA256 checksum be checked
    /// before cleaning the files
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    pub process_samples: bool,
    /// Output directory to store the cleaned reads
    pub output_dir: &'a Path,
    /// Optional parameters for the cleaning process
    pub optional_params: Option<&'a str>,
}

impl ReadCleaner<'_> {
    /// Initialize a new ReadCleaner instance
    pub fn new<'a>(args: &'a CleanArgs) -> ReadCleaner<'a> {
        ReadCleaner {
            config_path: &args.config,
            ignore_checksum: args.ignore_checksum,
            process_samples: args.process_samples,
            output_dir: &args.output,
            optional_params: args.optional_params.as_deref(),
        }
    }

    /// Clean raw read files using Fastp
    pub fn clean(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        let spinner = utils::init_spinner();
        spinner.set_message("Checking config for errors");
        let status = self.check_config(&config.samples);

        if !self.process_samples {
            spinner.finish_with_message("Finished checking config\n");
            self.log_config_check(&status);
            self.log_unprocessed();
            return;
        }

        spinner.set_message("Cleaning reads");
        self.clean_reads(&config.samples);

        self.print_summary();
    }

    fn parse_config(&self) -> Result<RawReadConfig, Box<dyn std::error::Error>> {
        let mut config = RawReadConfig::default();
        config.from_yaml(self.config_path)?;

        if config.sample_counts != config.samples.len() {
            return Err("Sample counts do not match the number of samples".into());
        }

        Ok(config)
    }

    fn clean_reads(&self, samples: &[FastqReads]) {
        samples.iter().for_each(|sample| {
            let runner = fastp::FastpRunner::new(sample, self.output_dir, self.optional_params);
            runner.run().expect("Failed to run fastp");
        });
    }

    fn check_config(&self, samples: &[FastqReads]) -> Vec<RawReadChecker> {
        let (tx, rx) = mpsc::channel();
        samples.par_iter().for_each_with(tx, |tx, sample| {
            let mut status = RawReadChecker::new(&sample.sample_name);
            status.check(&sample, self.ignore_checksum);
            tx.send(status).expect("Failed to send status");
        });

        rx.iter().collect()
    }

    fn log_config_check(&self, status: &[RawReadChecker]) {
        let error_free_samples = status.iter().filter(|s| s.is_error_free()).count();
        let samples_with_warnings = status.iter().filter(|s| s.has_warnings()).count();
        let samples_with_errors = status.iter().filter(|s| s.has_errors()).count();

        let count = format!("Summary of {} samples checked", status.len());
        let error_free = format!("{:18}: {}", "Error free", error_free_samples);

        log::info!("{}", count.cyan());
        log::info!("{}", error_free);

        if samples_with_warnings > 0 {
            let warnings = format!("{:18}: {}", "Warnings", samples_with_warnings);
            log::info!("{}", warnings.yellow());
        }

        if samples_with_errors > 0 {
            let errors = format!("{:18}: {}", "Errors", samples_with_errors);
            log::info!("{}", errors.red());
        }
    }

    fn log_unprocessed(&self) {
        let msg = format!(
            "\n{}\n{}: {}",
            "Samples were not processed",
            "To continue processing samples use",
            "ullar clean --process".green()
        );

        log::info!("{}", msg);
    }

    fn log_input(&self, config: &RawReadConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "config_file", self.config_path.display());
        log::info!("{:18}: {}", "sample_counts", config.sample_counts);
        log::info!("{:18}: {}\n", "file_counts", config.file_counts);
    }

    fn print_summary(&self) {
        unimplemented!("Print summary not implemented")
    }
}
