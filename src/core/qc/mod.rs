//! Clean raw read files using Fastp
pub mod fastp;

use std::path::Path;
use std::sync::mpsc;

use colored::Colorize;
use rayon::prelude::*;

use crate::cli::args::CleanArgs;
use crate::helper::reads::{FastqReads, RawReadChecker};
use crate::helper::utils;

use self::fastp::FastpReport;

use super::new::configs::RawReadConfig;

pub const DEFAULT_CLEAN_READ_OUTPUT_DIR: &str = "cleaned_reads";

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
    /// Check config for errors
    pub skip_config_check: bool,
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
            skip_config_check: args.skip_config_check,
        }
    }

    /// Clean raw read files using Fastp
    pub fn clean(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        let spinner = utils::init_spinner();
        spinner.set_message("Checking config for errors");
        let mut status = Vec::new();

        if self.skip_config_check {
            spinner.set_message("Skipping config check\n");
        } else {
            status = self.check_config(&config.samples);
        }

        if !self.process_samples {
            spinner.finish_with_message(format!("{} Finished checking config\n", "✔".green()));
            self.log_config_check(&status);
            self.log_unprocessed();
            return;
        }

        if !self.is_config_ok(&status) {
            self.log_config_check(&status);
        }

        spinner.set_message("Cleaning reads");
        let (succes_counts, failure_counts) = self.clean_reads(&config.samples);
        spinner.finish_with_message(format!("{} Finished cleaning reads\n", "✔".green()));
        self.print_final_summary(failure_counts, succes_counts);
    }

    fn is_config_ok(&self, status: &[RawReadChecker]) -> bool {
        status.iter().all(|s| s.is_error_free()) || status.is_empty()
    }

    fn parse_config(&self) -> Result<RawReadConfig, Box<dyn std::error::Error>> {
        let mut config = RawReadConfig::default();
        config.from_yaml(self.config_path)?;

        if config.sample_counts != config.samples.len() {
            return Err("Sample counts do not match the number of samples".into());
        }

        Ok(config)
    }

    fn clean_reads(&self, samples: &[FastqReads]) -> (usize, usize) {
        let success_counts = 0;
        let mut failure_counts = 0;
        samples.iter().for_each(|sample| {
            let mut runner = fastp::FastpRunner::new(sample, self.output_dir, self.optional_params);
            let reports = runner.run().expect("Failed to run fastp");

            match reports {
                Some(reports) => self.print_run_summary(&reports),
                None => {
                    failure_counts += 1;
                }
            }
        });

        (success_counts, failure_counts)
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

        let count = format!("\nSummary of {} samples checked", status.len());
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

    fn print_run_summary(&self, reports: &FastpReport) {
        log::info!("{:18}: {}", "Output directory", self.output_dir.display());
        log::info!("{:18}: {}", "HTML report", reports.html.display());
        log::info!("{:18}: {}", "JSON report", reports.json.display());
        log::info!("{:18}: {}", "Fastp log", reports.log.display());
    }

    fn print_final_summary(&self, success_counts: usize, failure_counts: usize) {
        let total_samples = failure_counts + success_counts;
        let success = format!("{:18}: {}", "Success", success_counts);
        let failure = format!("{:18}: {}", "Failure", failure_counts);
        let total = format!("{:18}: {}", "Total", total_samples);

        log::info!("\n{}", "Summary".cyan());
        log::info!("{}", total);
        log::info!("{}", success.green());
        log::info!("{}", failure.red());
    }
}
