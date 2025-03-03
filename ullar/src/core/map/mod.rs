//! Map contig to reference sequence
use std::{error::Error, path::Path};

use colored::Colorize;
use configs::ContigMappingConfig;
use lastz::{LastzMapping, DEFAULT_LASTZ_PARAMS};
use reports::MappingData;
use summary::FinalMappingSummary;
use writer::MappedContigWriter;

use crate::{
    cli::commands::map::MapContigArgs,
    helper::{common, files::PathCheck},
    types::{runner::RunnerOptions, Task},
};

pub mod configs;
pub mod exonerate;
pub mod init;
pub mod lastz;
pub mod minimap;
pub mod reports;
pub mod summary;
pub mod writer;

pub const DEFAULT_MAPPED_CONTIG_OUTPUT_DIR: &str = "mapped_contigs";
pub const DEFAULT_MAP_READ_OUTPUT_DIR: &str = "mapped_reads";

pub enum Aligner {
    Exonerate,
    Lastz,
    Minimap,
}

pub struct ContigMapping<'a> {
    pub config_path: &'a Path,
    pub output_dir: &'a Path,
    pub aligner: Aligner,
    pub runner: RunnerOptions<'a>,
    #[allow(dead_code)]
    task: Task,
}

impl<'a> ContigMapping<'a> {
    pub fn new(config_path: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            output_dir,
            aligner: Aligner::Lastz,
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn from_arg(args: &'a MapContigArgs) -> Self {
        Self {
            config_path: &args.config,
            output_dir: &args.output,
            aligner: Aligner::Lastz,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::ContigMapping,
        }
    }

    pub fn map(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Mapping contigs to reference sequence");
        let config = self.parse_config().expect("Failed to parse config");
        spinner.finish_with_message(format!("{} Finished parsing config\n", "✔".green()));
        self.log_input(config.contigs.len());
        PathCheck::new(self.output_dir, true, self.runner.force).prompt_exists(self.runner.dry_run);
        let results = self.run_lastz(&config);
        let summary = self.generate_mapped_contig(&results, &config);
        self.log_output(&results, &summary);
    }

    fn parse_config(&self) -> Result<ContigMappingConfig, Box<dyn Error>> {
        let config = ContigMappingConfig::from_toml(self.config_path)?;
        Ok(config)
    }

    fn run_lastz(&self, config: &ContigMappingConfig) -> Vec<MappingData> {
        let lastz = LastzMapping::new(
            &config.reference,
            self.output_dir,
            self.runner.override_args,
        );
        lastz.run(&config.contigs).expect("Failed to run Lastz")
    }

    fn generate_mapped_contig(
        &self,
        data: &[MappingData],
        config: &ContigMappingConfig,
    ) -> FinalMappingSummary {
        MappedContigWriter::new(data, self.output_dir, &config.reference).generate()
    }

    fn log_input(&self, file_count: usize) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config", self.config_path.display());
        log::info!("{:18}: {}", "File count", file_count);
        log::info!("{:18}: {}", "Task", self.task);
        self.log_aligner_info();
    }

    fn log_aligner_info(&self) {
        match self.aligner {
            Aligner::Lastz => log::info!("{:18}: {}", "Aligner:", "Lastz"),
            Aligner::Exonerate => log::info!("{:18}: {}", "Aligner:", "Exonerate"),
            Aligner::Minimap => log::info!("{:18}: {}", "Aligner:", "Minimap"),
        }
        match self.runner.override_args {
            Some(args) => log::info!("{:18}: {}\n", "Override args:", args),
            None => log::info!("{:18}: {}\n", "Override args:", DEFAULT_LASTZ_PARAMS),
        }
    }

    fn log_output(&self, report: &[MappingData], summary: &FinalMappingSummary) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Output dir", self.output_dir.display());
        log::info!("{:18}: {}", "Total processed", report.len());
        log::info!("{:18}: {}", "Reference counts", summary.total_references);
        log::info!("{:18}: {}", "Sample matches", summary.total_matches);
        log::info!(
            "{:18}: {:.2}%",
            "Percent coverage",
            summary.total_percent_coverage
        );
    }
}
