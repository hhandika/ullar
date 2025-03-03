//! Clean raw read files using Fastp
pub mod configs;
pub mod fastp;
pub mod init;
pub mod reports;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use colored::Colorize;
use comfy_table::Table;
use configs::{CleanReadConfig, DEFAULT_READ_CLEANING_CONFIG};

use self::reports::CleanReadReport;
use crate::cli::commands::clean::ReadCleaningArgs;
use crate::deps::check_dependency_match;
use crate::deps::fastp::FastpMetadata;
use crate::deps::DepMetadata;
use crate::helper::common;
use crate::helper::configs::{CONFIG_EXTENSION_TOML, DEFAULT_CONFIG_DIR};
use crate::helper::fastq::FastqConfigCheck;
use crate::helper::files::PathCheck;
use crate::helper::tracker::ProcessingTracker;
use crate::types::reads::FastqReads;
use crate::types::runner::RunnerOptions;
use crate::types::Task;

pub const DEFAULT_RAW_READS_DIR: &str = "raw_reads";
pub const DEFAULT_CLEAN_READ_OUTPUT_DIR: &str = "cleaned_reads";

pub struct ReadCleaner<'a> {
    /// Path to the raw read config file
    pub config_path: PathBuf,
    /// Output directory to store the cleaned reads
    pub output_dir: &'a Path,
    /// Runner options
    pub runner: RunnerOptions<'a>,
    task: Task,
}

impl<'a> ReadCleaner<'a> {
    /// Initialize a new ReadCleaner instance
    /// with the given parameters
    pub fn new<P: AsRef<Path>>(config_path: P, output_dir: &'a Path) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            output_dir,
            runner: RunnerOptions::default(),
            task: Task::CleanReads,
        }
    }

    /// Initialize a new ReadCleaner instance
    /// from command line arguments
    pub fn from_arg(args: &'a ReadCleaningArgs) -> Self {
        let config_path: PathBuf = match &args.config {
            Some(path) => path.to_owned(),
            None => PathBuf::from(DEFAULT_CONFIG_DIR)
                .join(DEFAULT_READ_CLEANING_CONFIG)
                .with_extension(CONFIG_EXTENSION_TOML),
        };

        Self {
            config_path,
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::CleanReads,
        }
    }

    /// Clean raw read files using Fastp
    pub fn clean(&self) {
        let config = self
            .parse_config()
            .expect("Failed to parse config. Try to create a new config.");
        self.log_input(&config);
        PathCheck::new(self.output_dir, true, self.runner.force).prompt_exists(self.runner.dry_run);
        let spinner = common::init_spinner();
        let mut check = FastqConfigCheck::new(config.input.sample_counts);
        if self.runner.skip_config_check {
            spinner.finish_with_message("Skipping config data check\n");
        } else {
            spinner.set_message("Checking config data for errors");
            check.check_fastq(&config.samples, self.runner.ignore_checksum);
            spinner.finish_with_message(format!("{} Finished checking config data\n", "✔".green()));
        }

        if self.runner.dry_run {
            check.log_status();
            self.log_unprocessed();
            return;
        }

        if !check.is_config_ok() && !self.runner.skip_config_check {
            check.log_status();
            log::error!("\n{}\n", "Config check failed".red());
            return;
        }

        let reports = self.clean_reads(&config.samples);
        // let config_path = self
        //     .write_output_config(&reports)
        //     .expect("Failed to write clean read config");
        self.log_final_output(&reports);
    }

    fn clean_reads(&self, samples: &[FastqReads]) -> Vec<CleanReadReport> {
        let mut tracker = ProcessingTracker::new(samples.len());
        let time = std::time::Instant::now();
        let mut reports = Vec::new();
        samples.iter().enumerate().for_each(|(i, sample)| {
            let mut runner =
                fastp::FastpRunner::new(sample, self.output_dir, self.runner.override_args);
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

    fn parse_config(&self) -> Result<CleanReadConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Input config path: {}", self.config_path.display()))?;
        let config: CleanReadConfig = toml::from_str(&content)?;

        if config.input.sample_counts != config.samples.len() {
            return Err("Sample counts do not match the number of samples".into());
        }

        Ok(config)
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

    fn log_input(&self, config: &CleanReadConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.input.sample_counts);
        log::info!("{:18}: {}", "File counts", config.input.file_counts);
        log::info!("{:18}: {}", "Task", self.task);
        self.log_fastp_info(&config.dependencies);
    }

    fn log_final_output(&self, reports: &[CleanReadReport]) {
        log::info!("{}", "\nOutput".cyan());
        log::info!("{:18}: {}", "Directory", self.output_dir.display());
        log::info!("{:18}: {}", "Total processed", reports.len());
    }

    fn log_fastp_info(&self, dependency: &DepMetadata) {
        let deps = FastpMetadata::new(None).get();
        match deps {
            Some(dep) => {
                log::info!("{:18}: {} v{}\n", "Cleaner", dep.name, dep.version);
                check_dependency_match(dependency, &dep.version);
            }
            None => panic!("Failed to find Fastp"),
        }
    }
}
