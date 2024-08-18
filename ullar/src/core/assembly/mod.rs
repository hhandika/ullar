use std::{error::Error, fs, path::Path, time::Instant};

use colored::Colorize;
use comfy_table::Table;
use configs::AssemblyConfig;
use spades::SpadeRunner;

use crate::{
    cli::commands::assembly::AssemblyArgs,
    helper::{common, fastq::FastqConfigCheck, files::PathCheck, tracker::ProcessingTracker},
    types::{reads::FastqReads, runner::RunnerOptions, Task},
};

use self::reports::SpadeReports;

use super::deps::spades::SpadesMetadata;

pub mod configs;
pub mod init;
pub mod reports;
pub mod spades;

pub const DEFAULT_ASSEMBLY_OUTPUT_DIR: &str = "assemblies";

pub struct Assembly<'a> {
    /// Path to the assembly configuration file
    pub config_path: &'a Path,
    /// Should the SHA256 checksum be checked
    /// before assembling the files
    pub ignore_checksum: bool,
    /// Output directory to store the assemblies
    pub output_dir: &'a Path,
    /// Remove SPAdes intermediate files
    /// by default
    pub keep_intermediates: bool,
    /// Rename contigs file to sample name
    pub rename_contigs: bool,
    /// Runner options
    pub runner: RunnerOptions<'a>,
    task: Task,
}

impl<'a> Assembly<'a> {
    /// Initialize a new Assembly instance
    /// with the given parameters
    pub fn new(
        config_path: &'a Path,
        ignore_checksum: bool,
        output_dir: &'a Path,
        keep_intermediates: bool,
        rename_contigs: bool,
    ) -> Self {
        Self {
            config_path,
            ignore_checksum,
            output_dir,
            keep_intermediates,
            rename_contigs,
            runner: RunnerOptions::default(),
            task: Task::Assembly,
        }
    }
    /// Initialize a new Assembly instance
    /// from the command line arguments
    pub fn from_arg(args: &'a AssemblyArgs) -> Self {
        Self {
            config_path: &args.config,
            ignore_checksum: args.common.ignore_checksum,
            output_dir: &args.output,
            keep_intermediates: args.keep_intermediates,
            rename_contigs: args.rename_contigs,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::Assembly,
        }
    }

    /// Assemble cleaned read files using SPAdes
    pub fn assemble(&self) {
        let config = self.parse_config().expect("Failed to parse config");
        self.log_input(&config);
        PathCheck::new(self.output_dir, true, self.runner.force).prompt_exists(self.runner.dry_run);
        let spinner = common::init_spinner();
        let mut check = FastqConfigCheck::new(config.sample_counts);
        if self.runner.skip_config_check {
            spinner.finish_with_message("Skipping config data check\n");
        } else {
            spinner.set_message("Checking config data for errors");
            check.check_fastq(&config.samples, self.ignore_checksum);
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

        let _ = self.assemble_reads(&config.samples);
        self.log_output();
    }

    fn parse_config(&self) -> Result<AssemblyConfig, Box<dyn Error>> {
        let content = fs::read_to_string(self.config_path)?;
        let config: AssemblyConfig = serde_yaml::from_str(&content)?;
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
                self.runner.override_args,
                self.keep_intermediates,
                self.rename_contigs,
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

    fn log_input(&self, config: &AssemblyConfig) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config path", self.config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.sample_counts);
        log::info!("{:18}: {}", "File counts", config.file_counts);
        log::info!("{:18}: {}", "Task", self.task);
        self.log_spade_info();
    }

    fn log_output(&self) {
        log::info!("{}", "Output summary".cyan());
        let output_dir = self.output_dir.join("assemblies");
        log::info!("{:18}: {}", "Output directory", output_dir.display());
    }

    fn log_spade_info(&self) {
        let deps = SpadesMetadata::new().get();
        match deps.metadata {
            Some(dep) => log::info!("{:18}: {} v{}\n", "Assembler", dep.name, dep.version),
            None => log::info!("{:18}: {}\n", "Assembler", "SPAdes".to_string()),
        }
    }
}
