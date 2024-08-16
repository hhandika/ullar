//! Map contig to reference sequence
use std::{error::Error, path::Path};

use colored::Colorize;
use configs::MappedContigConfig;
use lastz::LastzMapping;
use reports::MappingData;
use sequences::MappedContigs;

use crate::{
    cli::commands::map::MapContigArgs,
    helper::{common, files::FileMetadata},
    types::{runner::RunnerOptions, Task},
};

pub mod configs;
pub mod exonerate;
pub mod init;
pub mod lastz;
pub mod minimap;
pub mod reports;
pub mod sequences;

pub const DEFAULT_MAPPED_CONTIG_OUTPUT_DIR: &str = "mapped_contigs";
pub const DEFAULT_MAP_READ_OUTPUT_DIR: &str = "mapped_reads";

pub enum Aligner {
    Exonerate,
    Lastz,
    Minimap,
}

pub struct ContigMapping<'a> {
    pub config_path: &'a Path,
    pub reference: &'a Path,
    pub output_dir: &'a Path,
    pub aligner: Aligner,
    pub runner: RunnerOptions<'a>,
    #[allow(dead_code)]
    task: Task,
}

impl<'a> ContigMapping<'a> {
    pub fn new(config_path: &'a Path, reference: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            reference,
            output_dir,
            aligner: Aligner::Lastz,
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn from_arg(args: &'a MapContigArgs) -> Self {
        Self {
            config_path: &args.config,
            reference: &args.reference,
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
        self.log_input();
        let results = self.run_lastz(&config.contig_files);
        self.generate_mapped_contig(&results);
        self.log_output(&results);
    }

    fn parse_config(&self) -> Result<MappedContigConfig, Box<dyn Error>> {
        let config = std::fs::read_to_string(self.config_path)?;
        let config: MappedContigConfig = serde_yaml::from_str(&config)?;
        Ok(config)
    }

    fn run_lastz(&self, contigs: &[FileMetadata]) -> Vec<MappingData> {
        let runner =
            LastzMapping::new(&self.reference, &self.output_dir, self.runner.override_args);
        let report = runner.run(contigs).expect("Failed to run Lastz");
        report
    }

    fn generate_mapped_contig(&self, data: &[MappingData]) {
        MappedContigs::new(data, self.output_dir, self.reference).generate();
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18} {}", "Config:", self.config_path.display());
        log::info!("{:18} {}", "Reference:", self.reference.display());
        match self.aligner {
            Aligner::Lastz => log::info!("{:18} {}", "Aligner:", "Lastz"),
            Aligner::Exonerate => log::info!("{:18} {}", "Aligner:", "Exonerate"),
            Aligner::Minimap => log::info!("{:18} {}", "Aligner:", "Minimap"),
        }
    }

    fn log_output(&self, report: &[MappingData]) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18} {}", "Total contigs:", report.len());
        log::info!("{:18} {}", "Output dir:", self.output_dir.display());
    }
}
