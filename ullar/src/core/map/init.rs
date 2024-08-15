//! Initialize config file for mapping contigs to reference sequence.
//!
//! Include support for phyluce for UCE analysis workflow.
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    cli::commands::map::MapInitArgs, core::assembly::DEFAULT_ASSEMBLY_OUTPUT_DIR, helper::common,
    types::map::MappingQueryFormat,
};

use super::configs::MappedContigConfig;

pub struct InitMappingConfig<'a> {
    /// Target directory containing target reference sequences
    pub target_path: &'a Path,
    /// Query directory containing query sequences
    pub query_dir: &'a Path,
    /// Input query format
    pub query_format: MappingQueryFormat,
}

impl Default for InitMappingConfig<'_> {
    fn default() -> Self {
        Self {
            target_path: Path::new(DEFAULT_ASSEMBLY_OUTPUT_DIR),
            query_dir: Path::new(DEFAULT_ASSEMBLY_OUTPUT_DIR),
            query_format: MappingQueryFormat::Fasta,
        }
    }
}

impl<'a> InitMappingConfig<'a> {
    pub fn from_arg(args: &'a MapInitArgs) -> Self {
        Self {
            target_path: &args.target_dir,
            query_dir: &args.query_dir,
            query_format: args.query_format.parse().expect("Invalid query format"),
        }
    }

    pub fn init(&self) {
        let spinner = common::init_spinner();
        self.log_input();
        spinner.set_message("Initializing mapping configuration");
    }

    pub fn write_config(&self) -> Result<PathBuf, Box<dyn Error>> {
        match self.query_format {
            MappingQueryFormat::Fasta => {
                let mut config = MappedContigConfig::default();
                config.init(self.target_path, self.query_dir, None);
                if config.contig_files.is_empty() {
                    return Err(
                        "No sequence found in the input directory. Please, check input is FASTA"
                            .into(),
                    );
                }
                let output_path = config.to_yaml()?;
                Ok(output_path)
            }
            MappingQueryFormat::Fastq => Err("Fastq format is not supported yet".into()),
        }
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Target path", self.target_path.display());
        log::info!("{:18}: {}", "Query directory", self.query_dir.display());
        log::info!("{:18}: {}", "Task", "Initialize mapping config");
    }
}
