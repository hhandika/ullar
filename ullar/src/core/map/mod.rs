//! Map contig to reference sequence
use std::{error::Error, path::Path, sync::mpsc};

use colored::Colorize;
use configs::MappedContigConfig;
use lastz::{LastzQuery, LastzRunner, LastzTarget};
use rayon::prelude::*;
use reports::{ContigMappingResult, LastzReport};

use crate::{
    cli::commands::map::MapContigArgs,
    helper::{common, files::FileMetadata},
    types::{
        map::{LastzNameParse, LastzOutputFormat},
        runner::RunnerOptions,
        Task,
    },
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
    }

    fn parse_config(&self) -> Result<MappedContigConfig, Box<dyn Error>> {
        let config = std::fs::read_to_string(self.config_path)?;
        let config: MappedContigConfig = serde_yaml::from_str(&config)?;
        Ok(config)
    }

    fn run_lastz(&self, contigs: &[FileMetadata]) -> Vec<ContigMappingResult> {
        let progress_bar = common::init_progress_bar(contigs.len() as u64);
        log::info!("Mapping contigs to reference sequence");
        progress_bar.set_message("Contigs");
        let (tx, rx) = mpsc::channel();
        contigs.par_iter().for_each_with(tx, |tx, contig| {
            let runner = LastzRunner::new(contig, &self.reference, &self.output_dir);
            let report = runner
                .run(self.runner.override_args)
                .expect("Failed to run Lastz");
            tx.send(report).unwrap();
            progress_bar.inc(1);
        });
        let reports = rx.iter().collect::<Vec<ContigMappingResult>>();
        progress_bar.finish_with_message(format!("{} Contigs\n", "✔".green()));
        reports
    }
}
