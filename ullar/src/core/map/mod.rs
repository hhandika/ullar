//! Map contig to reference sequence
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;
use configs::{ContigMappingConfig, ReferenceFile, LASTZ_ALIGNER};
use lastz::{LastzMapping, DEFAULT_LASTZ_PARAMS};
use reports::MappingData;

use summary::FinalMappingSummary;
use writer::{LocusMappingWriter, ProbeMappingWriter};

use crate::{
    cli::commands::map::MapContigArgs,
    helper::{common, files::PathCheck},
    types::{
        map::{Aligner, LastzOutputFormat, MappingReferenceType},
        runner::RunnerOptions,
        Task,
    },
};

use super::deps::{lastz::LastzMetadata, DepMetadata};

pub mod configs;
pub mod exonerate;
pub mod init;
pub mod lastz;
pub mod minimap;
pub mod reports;
pub mod summary;
pub mod writer;

pub const DEFAULT_CONTIG_MAPPING_OUTPUT_DIR: &str = "out_contig_mapping";
pub const DEFAULT_READ_MAPPING_OUTPUT_DIR: &str = "out_read_mapping";

pub struct ContigMapping<'a> {
    pub config_path: &'a Path,
    pub output_dir: &'a Path,
    pub runner: RunnerOptions,
    task: Task,
}

impl<'a> ContigMapping<'a> {
    pub fn new(config_path: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            output_dir,
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn from_arg(args: &'a MapContigArgs) -> Self {
        Self {
            config_path: &args.config,
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::ContigMapping,
        }
    }

    pub fn from_config_path(config_path: &'a Path) -> Self {
        Self {
            config_path,
            output_dir: Path::new(DEFAULT_CONTIG_MAPPING_OUTPUT_DIR),
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn map(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Mapping contigs to reference sequence");
        let config = self.parse_config().expect("Failed to parse config");
        spinner.finish_with_message(format!("{} Finished parsing config\n", "✔".green()));
        let dep = config.dependencies.get(LASTZ_ALIGNER);
        let updated_dep = LastzMetadata::new().update(dep);
        self.log_input(config.contigs.len(), &config.input.aligner, &updated_dep);
        PathCheck::new(self.output_dir)
            .is_dir()
            .with_force_overwrite(self.runner.overwrite)
            .prompt_exists(self.runner.dry_run);
        self.run_lastz(&config, &updated_dep);
    }

    fn parse_config(&self) -> Result<ContigMappingConfig, Box<dyn Error>> {
        let config = ContigMappingConfig::from_toml(self.config_path)?;
        Ok(config)
    }

    fn run_lastz(&self, config: &ContigMappingConfig, dep: &DepMetadata) {
        let lastz = LastzMapping::new(&config.sequence_reference, self.output_dir, dep);

        match config.sequence_reference.reference_type {
            MappingReferenceType::Probes => {
                let lastz_output_fmt = LastzOutputFormat::General(String::new());
                let results = lastz
                    .map_to_probes(&config.contigs, &lastz_output_fmt)
                    .expect("Failed to run Lastz");
                let summary = self.write_probes_matched(&results, &config);
                self.log_output(results.len(), &summary);
            }
            MappingReferenceType::Loci => {
                let results = lastz
                    .map_to_reference(&config.contigs)
                    .expect("Failed to run Lastz");
                let summary = self.write_loci_matched(&results, &config.sequence_reference);
                self.log_output(results.len(), &summary);
            }
        }
    }

    fn write_loci_matched(
        &self,
        maf_files: &[PathBuf],
        reference: &ReferenceFile,
    ) -> FinalMappingSummary {
        LocusMappingWriter::new(self.output_dir, maf_files, reference).write()
    }

    fn write_probes_matched(
        &self,
        data: &[MappingData],
        config: &ContigMappingConfig,
    ) -> FinalMappingSummary {
        ProbeMappingWriter::new(self.output_dir, &config.sequence_reference).write_general(data)
    }

    fn log_input(&self, file_count: usize, aligner: &Aligner, dep: &DepMetadata) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config", self.config_path.display());
        log::info!("{:18}: {}", "File count", file_count);
        log::info!("{:18}: {}", "Task", self.task);
        self.log_aligner_info(aligner, dep);
    }

    fn log_aligner_info(&self, aligner: &Aligner, dep: &DepMetadata) {
        match aligner {
            Aligner::Lastz => log::info!("{:18}: {}", "Aligner:", "Lastz"),
            Aligner::Exonerate => log::info!("{:18}: {}", "Aligner:", "Exonerate"),
            Aligner::Minimap => log::info!("{:18}: {}", "Aligner:", "Minimap"),
        }
        match &dep.override_args {
            Some(args) => log::info!("{:18}: {}\n", "Override args:", args),
            None => log::info!("{:18}: {}\n", "Override args:", DEFAULT_LASTZ_PARAMS),
        }
    }

    fn log_output(&self, processed: usize, summary: &FinalMappingSummary) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Output dir", self.output_dir.display());
        log::info!("{:18}: {}", "Total processed", processed);
        log::info!("{:18}: {}", "Reference counts", summary.total_references);
        log::info!("{:18}: {}", "Sample matches", summary.total_matches);
        log::info!(
            "{:18}: {:.2}%",
            "Percent coverage",
            summary.total_percent_coverage
        );
    }
}
