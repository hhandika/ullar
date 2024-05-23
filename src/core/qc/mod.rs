//! Clean raw read files using Fastp
pub mod fastp;

use std::path::Path;
use std::sync::mpsc;

use colored::Colorize;
use comfy_table::Table;
use rayon::prelude::*;

use crate::cli::args::CleanArgs;
use crate::helper::reads::{FastqReads, RawReadChecker};
use crate::helper::tracker::ProcessingTracker;
use crate::helper::utils;

use self::fastp::FastpReport;

use super::configs::raw_reads::RawReadConfig;

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
        let mut status = Vec::new();

        if self.skip_config_check {
            spinner.set_message("Skipping config data check\n");
        } else {
            spinner.set_message("Checking config data for errors");
            status = self.check_config(&config.samples);
            spinner.finish_with_message(format!("{} Finished checking config data\n", "✔".green()));
        }

        if !self.process_samples {
            self.log_config_check(&status);
            self.log_unprocessed();
            return;
        }

        if !self.is_config_ok(&status) {
            self.log_config_check(&status);
            log::error!("\n{}\n", "Config check failed".red());
            return;
        }

        let tracker = self.clean_reads(&config.samples);
        tracker.finalize();
    }

    fn is_config_ok(&self, status: &[RawReadChecker]) -> bool {
        status.iter().all(|s| s.is_ok()) || status.is_empty()
    }

    fn parse_config(&self) -> Result<RawReadConfig, Box<dyn std::error::Error>> {
        let mut config = RawReadConfig::default();
        config.from_yaml(self.config_path)?;

        if config.sample_counts != config.samples.len() {
            return Err("Sample counts do not match the number of samples".into());
        }

        Ok(config)
    }

    fn clean_reads(&self, samples: &[FastqReads]) -> ProcessingTracker {
        let mut tracker = ProcessingTracker::new(samples.len());
        let time = std::time::Instant::now();
        samples.iter().enumerate().for_each(|(i, sample)| {
            let mut runner = fastp::FastpRunner::new(sample, self.output_dir, self.optional_params);
            let results = runner.run();

            match results {
                Ok(reports) => {
                    self.print_run_summary(&reports);
                    tracker.success_counts += 1;
                }
                Err(e) => {
                    log::error!("Failed to clean reads for sample: {}", sample.sample_name);
                    log::error!("{}", e);
                    tracker.failure_counts += 1;
                }
            }
            tracker.update(time.elapsed().as_secs_f64());

            if i < samples.len() - 1 {
                tracker.print_summary();
            }
        });

        tracker
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
        let ok_samples = status.iter().filter(|s| s.is_ok()).count();
        let samples_with_warnings = status.iter().filter(|s| s.has_warnings()).count();
        let samples_with_errors = status.iter().filter(|s| s.has_errors()).count();

        log::info!("{}", "Config check summary".cyan());
        log::info!("{:18}: {}", "Total samples", status.len());
        let ok_text = format!("{:18}: {}", "Pass", ok_samples);
        log::info!("{}", ok_text.green());

        if samples_with_warnings > 0 {
            log::info!("{:18}: {}", "Warnings".yellow(), samples_with_warnings);
        }

        if samples_with_errors > 0 {
            log::info!("{:18}: {}", "Errors".red(), samples_with_errors);
        }
    }

    fn log_unprocessed(&self) {
        let msg1 = "Samples were not processed";
        let msg2 = format!("To process samples use: {}", "ullar clean --process");
        let msg3 = format!(
            "To skip config check use: {}",
            "ullar clean --process --skip-config-check"
        );

        let mut table = Table::new();
        let text = format!("{}\n{}\n{}", msg1, msg2, msg3);
        table.add_row(vec![text]);
        log::info!("\n{}", table);
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
}
