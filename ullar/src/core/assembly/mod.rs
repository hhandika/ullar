use std::{
    error::Error,
    path::{Path, PathBuf},
    time::Instant,
};

use colored::Colorize;
use comfy_table::Table;
use configs::{AssemblyConfig, ASSEMBLER_DEPENDENCY, DEFAULT_ASSEMBLY_CONFIG};
use spades::SpadeRunner;

use crate::{
    cli::commands::assembly::AssemblyArgs,
    helper::{
        common,
        configs::{CONFIG_EXTENSION_TOML, DEFAULT_CONFIG_DIR},
        fastq::FastqConfigCheck,
        files::PathCheck,
        tracker::ProcessingTracker,
    },
    types::{reads::FastqReads, runner::RunnerOptions, Task},
};

use self::reports::SpadeReports;

use crate::core::deps::{spades::SpadesMetadata, DepMetadata};

pub mod configs;
pub mod init;
pub mod reports;
pub mod spades;

pub const DEFAULT_ASSEMBLY_OUTPUT_DIR: &str = "out_denovo_assembly";

pub struct Assembly<'a> {
    /// Path to the assembly config file
    pub config_path: PathBuf,
    /// Output directory to store the assemblies
    pub output_dir: &'a Path,
    /// Remove SPAdes intermediate files
    /// by default
    pub keep_intermediates: bool,
    /// Rename contigs file to sample name
    pub rename_contigs: bool,
    /// Runner options
    pub runner: RunnerOptions,
    task: Task,
}

impl<'a> Assembly<'a> {
    /// Initialize a new Assembly instance
    /// with the given parameters
    pub fn new<P: AsRef<Path>>(
        config_path: P,
        output_dir: &'a Path,
        keep_intermediates: bool,
        rename_contigs: bool,
    ) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
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
        let config_path = match &args.config {
            Some(path) => path.to_owned(),
            None => PathBuf::from(DEFAULT_CONFIG_DIR)
                .join(DEFAULT_ASSEMBLY_CONFIG)
                .with_extension(CONFIG_EXTENSION_TOML),
        };
        Self {
            config_path,
            output_dir: &args.output,
            keep_intermediates: args.keep_intermediates,
            rename_contigs: args.rename_contigs,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::Assembly,
        }
    }

    pub fn from_config_path(config_path: &Path) -> Self {
        Self {
            config_path: config_path.to_owned(),
            output_dir: Path::new(DEFAULT_ASSEMBLY_OUTPUT_DIR),
            rename_contigs: false,
            keep_intermediates: false,
            runner: RunnerOptions::default(),
            task: Task::Assembly,
        }
    }

    /// Assemble cleaned read files using SPAdes
    pub fn assemble(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Parsing config file\n");
        let config = self.parse_config().expect("Failed to parse config");
        let assembler = config.dependencies.get(ASSEMBLER_DEPENDENCY);
        let updated_dep = SpadesMetadata::new().update(assembler);
        self.log_input(&config, &updated_dep);
        PathCheck::new(self.output_dir)
            .is_dir()
            .with_force_overwrite(self.runner.overwrite)
            .prompt_exists(self.runner.dry_run);
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

        let _ = self.assemble_reads(&config.samples, &updated_dep);
        self.log_output();
    }

    fn parse_config(&self) -> Result<AssemblyConfig, Box<dyn Error>> {
        let config: AssemblyConfig = AssemblyConfig::from_toml(&self.config_path)?;
        Ok(config)
    }

    fn assemble_reads(&self, samples: &[FastqReads], dep: &DepMetadata) -> Vec<SpadeReports> {
        let time = Instant::now();
        let mut tracker = ProcessingTracker::new(samples.len());
        let mut reports = Vec::new();

        samples.iter().enumerate().for_each(|(idx, sample)| {
            let results = SpadeRunner::new(sample, self.output_dir, dep)
                .keep_intermediates(self.keep_intermediates)
                .rename_contigs(self.rename_contigs)
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

    fn log_input(&self, config: &AssemblyConfig, dep: &DepMetadata) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config path", self.config_path.display());
        config.input.log_summary();
        log::info!("{:18}: {}", "Task", self.task);
        log::info!("{:18}: {} v{}\n", "Assembler", dep.app_name, dep.version);
    }

    fn log_output(&self) {
        log::info!("{}", "Output summary".cyan());
        let output_dir = self.output_dir.join("assemblies");
        log::info!("{:18}: {}", "Output directory", output_dir.display());
    }
}
