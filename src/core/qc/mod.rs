//! Clean raw read files using Fastp
pub mod fastp;

use std::path::Path;
use std::sync::mpsc;

use colored::Colorize;
use indicatif::ProgressBar;
use rayon::prelude::*;

use crate::helper::reads::{FastqReads, RawReadChecker};
use crate::helper::utils;

use super::new::configs::RawReadConfig;

pub struct ReadCleaner<'a> {
    /// Path to the raw read configuration file
    pub config_path: &'a Path,
    /// Should the SHA256 checksum be checked
    /// before cleaning the files
    pub check_sha256: bool,
    /// Process samples if true
    /// else check the config file only
    pub process_samples: bool,
}

impl ReadCleaner<'_> {
    /// Initialize a new ReadCleaner instance
    pub fn new<'a>(config_path: &'a Path, check_sha256: bool) -> ReadCleaner<'a> {
        ReadCleaner {
            config_path,
            check_sha256,
            process_samples: true,
        }
    }

    /// Clean raw read files using Fastp
    pub fn clean(&self) {
        let spinner = utils::init_spinner();
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        let status = self.check_config(&config.samples, &spinner);
        spinner.finish_with_message("Finished checking config");
        if !self.process_samples {
            self.log_config_check(&status);
            self.log_unprocessed();
            return;
        }

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

    fn check_config(&self, samples: &[FastqReads], spinner: &ProgressBar) -> Vec<RawReadChecker> {
        let (tx, rx) = mpsc::channel();
        samples.par_iter().for_each_with(tx, |tx, sample| {
            spinner.set_message(format!("Checking {}", sample.sample_name));
            let mut status = RawReadChecker::new(&sample.sample_name);
            status.check(&sample);
            tx.send(status).expect("Failed to send status");
        });

        rx.iter().collect()
    }

    fn log_config_check(&self, status: &[RawReadChecker]) {
        let error_free_samples = status.iter().filter(|s| s.is_error_free()).count();
        let samples_with_warnings = status.iter().filter(|s| s.has_warnings()).count();
        let samples_with_errors = status.iter().filter(|s| s.has_errors()).count();

        log::info!("{:18}: {}", "Error free samples", error_free_samples);
        log::info!("{:18}: {}", "Samples with warnings", samples_with_warnings);
        log::info!("{:18}: {}", "Samples with errors", samples_with_errors);
    }

    fn log_unprocessed(&self) {
        let msg1 = "Samples not processed";
        let msg2 = format!(
            "To continue processing samples: {}",
            "ullar clean --process".green()
        );
        log::info!("{}", msg1);
        log::info!("{}", msg2);
    }

    fn log_input(&self, config: &RawReadConfig) {
        log::info!("{:18}: {}", "sample_counts", config.sample_counts);
        log::info!("{:18}: {}", "file_counts", config.file_counts);
    }

    fn print_summary(&self) {
        unimplemented!("Print summary not implemented")
    }
}
