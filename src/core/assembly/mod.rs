use std::{fs, path::Path, time::Instant};

use colored::Colorize;
use comfy_table::Table;
use spades::SpadeRunner;

use crate::{
    cli::args::AssemblyArgs,
    helper::{common, files::PathCheck, reads::FastqReads, tracker::ProcessingTracker},
};

use self::reports::SpadeReports;

use super::configs::{cleaned_reads::CleanReadConfig, ConfigCheck};

pub mod reports;
pub mod spades;

pub const DEFAULT_ASSEMBLY_OUTPUT_DIR: &str = "assemblies";

pub struct Assembly<'a> {
    /// Path to the assembly configuration file
    pub config_path: &'a Path,
    /// Should the SHA256 checksum be checked
    /// before assembling the files
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    pub process_samples: bool,
    /// Output directory to store the assemblies
    pub output_dir: &'a Path,
    /// Optional parameters for the assembly process
    pub optional_params: Option<&'a str>,
    /// Check config for errors
    pub skip_config_check: bool,
    /// Remove SPAdes intermediate files
    /// by default
    pub keep_intermediates: bool,
}

impl Assembly<'_> {
    /// Initialize a new Assembly instance
    pub fn new<'a>(args: &'a AssemblyArgs) -> Assembly<'a> {
        Assembly {
            config_path: &args.config,
            ignore_checksum: args.ignore_checksum,
            process_samples: args.process_samples,
            output_dir: &args.output,
            optional_params: args.optional_params.as_deref(),
            skip_config_check: args.skip_config_check,
            keep_intermediates: args.keep_intermediates,
        }
    }

    /// Assemble cleaned read files using SPAdes
    pub fn assemble(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        if self.process_samples {
            PathCheck::new(self.output_dir, true).prompt_exists();
        }
        let spinner = common::init_spinner();
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

        let reports = self.assemble_reads(&config.samples);
        self.log_output(&reports);
    }

    fn parse_config(&self) -> Result<CleanReadConfig, Box<dyn std::error::Error>> {
        let config = fs::read_to_string(self.config_path)?;
        let config: CleanReadConfig = serde_yaml::from_str(&config)?;
        Ok(config)
    }

    fn assemble_reads(&self, samples: &[FastqReads]) -> Vec<SpadeReports> {
        let time = Instant::now();
        let mut tracker = ProcessingTracker::new(samples.len());
        let mut reports = Vec::new();

        samples.iter().enumerate().for_each(|(idx, sample)| {
            let results = SpadeRunner::new(
                sample,
                self.output_dir,
                self.optional_params,
                self.keep_intermediates,
            )
            .run();

            match results {
                Ok(report) => {
                    reports.push(report);
                    tracker.success_counts += 1;
                }
                Err(e) => {
                    log::error!("Failed to assemble sample: {}", sample.sample_name);
                    log::error!("Error: {}", e);
                    tracker.failure_counts += 1;
                }
            }

            tracker.update(time.elapsed().as_secs_f64());
            if idx < samples.len() - 1 {
                tracker.print_summary();
            }
        });

        tracker.finalize();
        reports
    }

    fn log_unprocessed(&self) {
        let msg1 = "Samples were not processed";
        let msg2 = format!("To process samples use: {}", "ullar assemble --process");
        let msg3 = format!(
            "To skip config check use: {}",
            "ullar assemble --process --skip-config-check"
        );

        let mut table = Table::new();
        let text = format!("{}\n{}\n{}", msg1, msg2, msg3);
        table.add_row(vec![text]);
        log::info!("\n{}", table);
    }

    fn log_input(&self, config: &CleanReadConfig) {
        log::info!("{}", "Assembly input".cyan());
        log::info!("{:18}: {}", "Config file", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.sample_counts);
        log::info!("{:18}: {}\n", "File counts", config.file_counts);
    }

    fn log_output(&self, reports: &[SpadeReports]) {
        log::info!("{}", "Assembly summary".cyan());
        let output_dir = self.output_dir.join("assemblies");
        log::info!("{:18}: {}", "Output directory", output_dir.display());
        log::info!("{:18}: {}", "Total samples", reports.len());
    }
}
