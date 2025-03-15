//! Initialize config file for mapping contigs to reference sequence.
//!
//! Include support for phyluce for UCE analysis workflow.
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    cli::commands::{common::CommonInitArgs, map::MapInitArgs},
    core::map::ContigMapping,
    helper::{
        common,
        configs::{CONFIG_EXTENSION_TOML, DEFAULT_CONFIG_DIR},
        files::PathCheck,
    },
    types::map::MappingQueryFormat,
};

use super::configs::{ContigInput, ContigMappingConfig, SampleNameSource};

pub struct InitMappingConfig<'a> {
    /// Query directory containing query sequences
    pub query_dir: Option<&'a Path>,
    /// Query file paths
    pub query_paths: Option<&'a [PathBuf]>,
    /// Input query format
    pub query_format: MappingQueryFormat,
    /// Source to parse file names
    /// Path to reference sequence
    pub reference_path: &'a Path,
    pub name_source: &'a str,
    /// Config file name
    pub config_name: &'a str,
    /// Reference regex names
    pub refname_regex: &'a str,
    /// Sample name regex
    pub sample_name_regex: &'a str,
    pub common: &'a CommonInitArgs,
}

impl<'a> InitMappingConfig<'a> {
    pub fn from_arg(args: &'a MapInitArgs) -> Self {
        Self {
            query_dir: args.dir.as_deref(),
            query_paths: args.input.as_deref(),
            query_format: args.query_format.parse().expect("Invalid query format"),
            reference_path: &args.reference,
            name_source: &args.name_source,
            config_name: &args.config_name,
            refname_regex: &args.re_reference,
            sample_name_regex: &args.re_sample,
            common: &args.common,
        }
    }

    pub fn init(&self) {
        self.log_input();
        let config_path = Path::new(DEFAULT_CONFIG_DIR)
            .join(self.config_name)
            .with_extension(CONFIG_EXTENSION_TOML);
        PathCheck::new(&config_path).prompt_exists(false);
        let spinner = common::init_spinner();
        spinner.set_message("Writing mapping config");
        let config_path = self.write_config();
        match config_path {
            Ok(path) => {
                spinner
                    .finish_with_message(format!("{} Finished writing config file\n", "✔".green()));
                self.log_output(&path);

                self.autorun_pipeline(&path);
            }
            Err(e) => {
                spinner.finish_with_message(format!(
                    "{} Failed to write config file: {}\n",
                    "✖".red(),
                    e
                ));
            }
        }
    }

    fn autorun_pipeline(&self, config_path: &Path) {
        let header = "Starting mapping pipeline...".to_string();
        log::info!("{}", header.cyan());
        log::info!("");

        let runner = ContigMapping::from_config_path(config_path);
        runner.map();
    }

    fn write_config(&self) -> Result<PathBuf, Box<dyn Error>> {
        match self.query_format {
            MappingQueryFormat::Contig => {
                let (path, config) = self.write_contig_config()?;
                self.log_output(&path);
                self.log_contig_output(&config);
                Ok(path)
            }
            MappingQueryFormat::Fastq => unimplemented!(),
        }
    }

    fn write_contig_config(&self) -> Result<(PathBuf, ContigMappingConfig), Box<dyn Error>> {
        let name_source = self.get_sample_name_source();
        let input = ContigInput::new(name_source);
        let mut config = ContigMappingConfig::init(input, self.refname_regex);
        match self.query_dir {
            Some(dir) => config.from_contig_dir(dir),
            None => config.from_contig_paths(&self.get_contig_paths()),
        }
        if config.contigs.is_empty() {
            return Err(
                "No sequence found in the input directory. Please, check input is FASTA".into(),
            );
        }
        let output_path = config.to_toml(
            self.config_name,
            self.reference_path,
            self.common.override_args.as_deref(),
        )?;
        Ok((output_path, config))
    }

    fn get_sample_name_source(&self) -> SampleNameSource {
        let mut source = self
            .name_source
            .parse::<SampleNameSource>()
            .expect("Invalid name source");
        if let SampleNameSource::Regex(_) = source {
            source = SampleNameSource::Regex(self.sample_name_regex.to_string());
        }
        source
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

    fn log_contig_output(&self, config: &ContigMappingConfig) {
        log::info!("{:18}: {}", "Sample counts", config.input.file_counts);
    }
}
