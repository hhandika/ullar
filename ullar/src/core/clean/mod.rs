//! Clean raw read files using Fastp
pub mod fastp;
pub mod reports;

use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use comfy_table::Table;

use crate::cli::commands::clean::CleanArgs;
use crate::core::configs::FastqConfigCheck;
use crate::helper::common;
use crate::helper::files::PathCheck;
use crate::helper::tracker::ProcessingTracker;
use crate::types::reads::FastqReads;
use crate::types::Task;

use self::reports::CleanReadReport;

use super::configs::cleaned_reads::CleanReadConfig;
use super::configs::raw_reads::RawReadConfig;
use super::utils::deps::FastpMetadata;

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
    task: Task,
}

impl<'a> ReadCleaner<'a> {
    /// Initialize a new ReadCleaner instance
    /// with the given parameters
    pub fn new(
        config_path: &'a Path,
        ignore_checksum: bool,
        process_samples: bool,
        output_dir: &'a Path,
        optional_params: Option<&'a str>,
        skip_config_check: bool,
    ) -> Self {
        Self {
            config_path,
            ignore_checksum,
            process_samples,
            output_dir,
            optional_params,
            skip_config_check,
            task: Task::CleanReads,
        }
    }

    /// Initialize a new ReadCleaner instance
    /// from command line arguments
    pub fn from_arg(args: &'a CleanArgs) -> Self {
        Self {
            config_path: &args.config,
            ignore_checksum: args.common.ignore_checksum,
            process_samples: args.common.process,
            output_dir: &args.output,
            optional_params: args.common.override_args.as_deref(),
            skip_config_check: args.common.skip_config_check,
            task: Task::CleanReads,
        }
    }

    /// Clean raw read files using Fastp
    pub fn clean(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        if self.process_samples {
            PathCheck::new(self.output_dir, true).prompt_exists();
        }
        let spinner = common::init_spinner();
        let mut check = FastqConfigCheck::new(config.sample_counts);
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
        let config_path = self
            .write_output_config(&reports)
            .expect("Failed to write clean read config");
        self.log_final_output(&config_path);
    }

    fn clean_reads(&self, samples: &[FastqReads]) -> Vec<CleanReadReport> {
        let mut tracker = ProcessingTracker::new(samples.len());
        let time = std::time::Instant::now();
        let mut reports = Vec::new();
        samples.iter().enumerate().for_each(|(i, sample)| {
            let mut runner = fastp::FastpRunner::new(sample, self.output_dir, self.optional_params);
            let results = runner.run();

            match results {
                Ok(report) => {
                    reports.push(report);
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

        tracker.finalize();
        reports
    }

    fn parse_config(&self) -> Result<RawReadConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: RawReadConfig = serde_yaml::from_str(&content)?;

        if config.sample_counts != config.samples.len() {
            return Err("Sample counts do not match the number of samples".into());
        }

        Ok(config)
    }

    // Prepare config for De Novo Assembly
    fn write_output_config(
        &self,
        reports: &[CleanReadReport],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let spin = common::init_spinner();
        spin.set_message("Writing output config");
        let fastp_dep = FastpMetadata::new().get();
        let mut metadata = Vec::new();

        if let Some(fastp) = fastp_dep.metadata {
            metadata.push(fastp);
        }
        let mut config = CleanReadConfig::new(
            Some(self.config_path.to_path_buf()),
            self.output_dir,
            metadata,
            self.optional_params.map(|s| s.to_string()),
        );

        let output = config.to_yaml(reports);
        spin.finish_with_message(format!("{} Finished writing output config\n", "✔".green()));
        output
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
        log::info!("{:18}: {}", "File counts", config.file_counts);
        log::info!("{:18}: {}", "Task", self.task);
        self.log_fastp_info();
    }

    fn log_final_output(&self, config_path: &Path) {
        log::info!("{}", "\nOutput".cyan());
        log::info!("{:18}: {}", "Directory", self.output_dir.display());
        log::info!("{:18}: {}\n", "Config file", config_path.display());
    }

    fn log_fastp_info(&self) {
        let deps = FastpMetadata::new().get();
        match deps.metadata {
            Some(dep) => log::info!("{:18}: {} v{}\n", "Cleaner", dep.name, dep.version),
            None => log::info!("{:18}: {}\n", "Cleaner", "fastp"),
        }
    }
}
