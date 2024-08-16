//! Map contig to reference sequence
use std::{error::Error, path::Path};

use colored::Colorize;
use configs::MappedContigConfig;
use lastz::LastzRunner;
use reports::LastzReport;

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
        let results = self.run_lastz(&config.contig_files);
        self.log_output(&results);
    }

    fn parse_config(&self) -> Result<MappedContigConfig, Box<dyn Error>> {
        let config = std::fs::read_to_string(self.config_path)?;
        let config: MappedContigConfig = serde_yaml::from_str(&config)?;
        Ok(config)
    }

    fn run_lastz(&self, contigs: &[FileMetadata]) -> Vec<LastzReport> {
        let runner = LastzRunner::new(&self.reference, &self.output_dir, self.runner.override_args);
        let report = runner.run(contigs).expect("Failed to run Lastz");
        report
    }

    fn log_output(&self, report: &[LastzReport]) {
        report.iter().for_each(|r| {
            println!("Output path: {}", r.output_path.display());
        });
    }
}
