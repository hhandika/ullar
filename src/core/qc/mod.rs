//! Clean raw read files using Fastp
pub mod fastp;

use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use comfy_table::Table;

use crate::cli::args::CleanArgs;
use crate::core::configs::ConfigCheck;
use crate::helper::files::PathCheck;
use crate::helper::reads::FastqReads;
use crate::helper::tracker::ProcessingTracker;
use crate::helper::utils;

use self::fastp::FastpReport;

use super::configs::cleaned_reads::CleanReadConfig;
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
        PathCheck::new(self.output_dir, true).prompt_exists();
        let spinner = utils::init_spinner();
        let mut check = ConfigCheck::new(config.sample_counts);

        if self.skip_config_check {
            spinner.finish_with_message("Skipping config data check\n");
        } else {
            spinner.set_message("Checking config data for errors");
            check.check_fastq(&config.samples, self.ignore_checksum);
            spinner.finish_with_message(format!("{} Finished checking config data\n", "✔".green()));
        }

        if !self.process_samples {
            check.log_status();
            self.log_unprocessed();
            return;
        }

        if !check.is_config_ok() && !self.skip_config_check {
            check.log_status();
            log::error!("\n{}\n", "Config check failed".red());
            return;
        }

        let reports = self.clean_reads(&config.samples);

        log::info!("{}", "Cleaning summary".cyan());
        let config_path = self
            .write_output_config(&reports)
            .expect("Failed to write clean read config");
        self.log_final_output(&config_path);
    }

    fn parse_config(&self) -> Result<RawReadConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: RawReadConfig = serde_yaml::from_str(&content)?;

        if config.sample_counts != config.samples.len() {
            return Err("Sample counts do not match the number of samples".into());
        }

        Ok(config)
    }

    fn write_output_config(
        &self,
        reports: &[FastpReport],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = self.output_dir.join(DEFAULT_CLEAN_READ_OUTPUT_DIR);
        let mut config = CleanReadConfig::new(
            Some(self.config_path.to_path_buf()),
            self.output_dir,
            Vec::new(),
            self.optional_params.map(|s| s.to_string()),
        );
        config.to_yaml(&output_dir, reports)
    }

    fn clean_reads(&self, samples: &[FastqReads]) -> Vec<FastpReport> {
        let mut tracker = ProcessingTracker::new(samples.len());
        let time = std::time::Instant::now();
        let mut reports = Vec::new();
        samples.iter().enumerate().for_each(|(i, sample)| {
            let mut runner = fastp::FastpRunner::new(sample, self.output_dir, self.optional_params);
            let results = runner.run();

            match results {
                Ok(report) => {
                    self.log_run_summary(&report);
                    tracker.success_counts += 1;
                    reports.push(report);
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

        tracker.finalize();
        reports
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
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.sample_counts);
        log::info!("{:18}: {}\n", "File counts", config.file_counts);
    }

    fn log_run_summary(&self, reports: &FastpReport) {
        log::info!("{:18}: {}", "Output directory", self.output_dir.display());
        log::info!("{:18}: {}", "HTML report", reports.html.display());
        log::info!("{:18}: {}", "JSON report", reports.json.display());
        log::info!("{:18}: {}", "Fastp log", reports.log.display());
    }

    fn log_final_output(&self, config_path: &Path) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Directory", self.output_dir.display());
        log::info!("{:18}: {}\n", "Config file", config_path.display());
    }
}
