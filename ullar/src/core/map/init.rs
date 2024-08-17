//! Initialize config file for mapping contigs to reference sequence.
//!
//! Include support for phyluce for UCE analysis workflow.
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;
use indicatif::ProgressBar;

use crate::{cli::commands::map::MapInitArgs, helper::common, types::map::MappingQueryFormat};

use super::configs::{MappedContigConfig, SampleNameSource};

pub struct InitMappingConfig<'a> {
    /// Query directory containing query sequences
    pub query_dir: Option<&'a Path>,
    /// Target directory containing target reference sequences
    pub query_paths: Option<&'a [PathBuf]>,
    /// Input query format
    pub query_format: MappingQueryFormat,
    /// Source to parse file names
    pub name_source: &'a str,
}

impl Default for InitMappingConfig<'_> {
    fn default() -> Self {
        Self {
            query_dir: None,
            query_paths: None,
            query_format: MappingQueryFormat::Contig,
            name_source: "file",
        }
    }
}

impl<'a> InitMappingConfig<'a> {
    pub fn from_arg(args: &'a MapInitArgs) -> Self {
        Self {
            query_dir: args.dir.as_deref(),
            query_paths: args.input.as_deref(),
            query_format: args.query_format.parse().expect("Invalid query format"),
            name_source: &args.name_source,
        }
    }

    pub fn init(&self) {
        self.log_input();
        let spinner = common::init_spinner();
        spinner.set_message("Initializing mapping configuration");
        self.write_config(&spinner);
    }

    fn write_config(&self, spinner: &ProgressBar) {
        match self.query_format {
            MappingQueryFormat::Contig => {
                spinner.set_message("Writing contig config");
                let (path, config) = self.write_contig_config().expect("Failed writing config");
                spinner.finish_with_message(format!("{} Finished writing config\n", "✔".green()));
                self.log_output(&path);
                self.log_contig_output(&config);
            }
            MappingQueryFormat::Fastq => unimplemented!(),
        }
    }

    fn write_contig_config(&self) -> Result<(PathBuf, MappedContigConfig), Box<dyn Error>> {
        let name_source = self
            .name_source
            .parse::<SampleNameSource>()
            .expect("Invalid name source");
        let mut config = MappedContigConfig::init(name_source);
        match self.query_dir {
            Some(dir) => config.from_contig_dir(dir, None),
            None => config.from_contig_paths(&self.get_contig_paths(), None),
        }
        if config.contigs.is_empty() {
            return Err(
                "No sequence found in the input directory. Please, check input is FASTA".into(),
            );
        }
        let output_path = config.to_yaml()?;
        Ok((output_path, config))
    }

    fn get_contig_paths(&self) -> Vec<PathBuf> {
        match self.query_paths {
            Some(paths) => {
                if paths.is_empty() {
                    panic!("No contig files found in input");
                }
                paths.to_vec()
            }
            None => panic!("No directory found"),
        }
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        match self.query_dir {
            Some(dir) => {
                log::info!("{:18}: {}", "Directory", dir.display());
            }
            None => self.log_input_paths(),
        }
        log::info!("{:18}: {}", "Format", self.query_format);
        log::info!("{:18}: {}", "Name source", self.name_source);
        log::info!("{:18}: {}\n", "Task", "Initialize mapping config");
    }

    fn log_input_paths(&self) {
        match self.query_paths {
            Some(paths) => {
                log::info!("{:18}: {}", "Input path", "Stdin");
                log::info!("{:18}: {}", "File counts", paths.len());
            }
            None => {
                log::info!("{:18}: {}", "Input directory", "Multiple files");
            }
        }
    }

    fn log_output(&self, output_path: &Path) {
        log::info!("{}", "Output".cyan());
        log::info!(
            "{:18}: {}",
            "Directory",
            output_path
                .parent()
                .expect("Failed parsing parent dir")
                .display()
        );
        log::info!(
            "{:18}: {}",
            "File",
            output_path
                .file_name()
                .expect("Failed parsing file")
                .to_str()
                .expect("Failed parsing file")
        );
    }

    fn log_contig_output(&self, config: &MappedContigConfig) {
        log::info!("{:18}: {}", "Contig counts", config.contig_file_counts);
    }
}
