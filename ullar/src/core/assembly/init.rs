//! Initialize config file for assembly workflow.
//!
//! It allows ullar to generate a config file when previous workflow
//! is done using other tools.

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    cli::commands::{
        assembly::AssemblyInitArgs,
        common::{CommonInitArgs, GenomicReadsInitArgs},
    },
    helper::{
        common,
        fastq::{FastqInput, ReadAssignmentStrategy},
        files::FileFinder,
    },
    types::{
        reads::{FastqReads, ReadAssignment, SampleNameFormat},
        SupportedFormats,
    },
};

use super::configs::AssemblyConfig;

pub struct AssemblyInit<'a> {
    input_dir: &'a Path,
    common: &'a CommonInitArgs,
    reads: &'a GenomicReadsInitArgs,
    sample_name_format: SampleNameFormat,
}

impl<'a> AssemblyInit<'a> {
    pub fn from_arg(args: &'a AssemblyInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            common: &args.common,
            reads: &args.reads,
            sample_name_format: args
                .reads
                .sample_name
                .parse::<SampleNameFormat>()
                .expect("Invalid sample name format"),
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.log_input();
        let spin = common::init_spinner();
        spin.set_message("Finding files...");
        let format = SupportedFormats::Fastq;
        self.match_sample_name_format();
        let files = FileFinder::new(self.input_dir, &format).find(self.reads.recursive)?;

        if files.is_empty() {
            spin.finish_with_message(format!(
                "{} No files found in {}!",
                "✖".red(),
                self.input_dir.display()
            ));
            self.log_empty_input();
            return Ok(());
        }

        let file_count = files.len();
        spin.set_message(format!(
            "Found {} files. Assigning reads and generating hash for matching files...",
            file_count
        ));

        let samples = self.assign_reads(&files);
        let sample_count = samples.len();
        spin.set_message(format!(
            "Found {} samples of {} files. Writing config file...",
            sample_count, file_count
        ));

        let config_path = self.write_config(samples, file_count)?;
        spin.finish_with_message(format!("{} Finished creating a config file\n", "✔".green()));
        self.log_output(&config_path, sample_count, file_count);
        Ok(())
    }

    fn log_empty_input(&self) {
        if !self.input_dir.exists() {
            log::error!(
                "\nInput directory does not exist: {}",
                self.input_dir.display()
            );
            log::error!("Use the --dir arg to specify a valid directory.");
        } else {
            log::error!(
                "\nTry to use --recursive flag \
                to search for files in subdirectories of {}",
                self.input_dir.display()
            );
        }
    }

    fn match_sample_name_format(&mut self) {
        if let Some(regex) = &self.reads.re_sample {
            self.sample_name_format = SampleNameFormat::Custom(regex.to_string());
        }
    }

    fn assign_reads(&self, files: &[PathBuf]) -> Vec<FastqReads> {
        ReadAssignment::new(files, &self.sample_name_format).assign()
    }

    fn write_config(
        &self,
        records: Vec<FastqReads>,
        file_counts: usize,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let strategy = ReadAssignmentStrategy::from_arg(self.reads);
        let input_summary = FastqInput::new(self.input_dir, records.len(), file_counts, strategy);
        let mut config = AssemblyConfig::new(input_summary, records.to_vec());
        let output_path = config.to_toml(self.common.override_args.as_deref())?;
        Ok(output_path)
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.input_dir.display());
        log::info!("{:18}: {}\n", "Sample name format", self.reads.sample_name);
    }

    fn log_output(&self, output_path: &Path, record_counts: usize, file_counts: usize) {
        let config_filename = output_path
            .file_name()
            .expect("Failed to get config file name")
            .to_string_lossy();
        log::info!("{}", "Output".cyan());
        log::info!(
            "{:18}: {}",
            "Directory",
            output_path
                .parent()
                .unwrap_or(Path::new("Unknown"))
                .display()
        );
        log::info!("{:18}: {}", "Config file", config_filename);
        log::info!("{:18}: {}", "Sample counts", record_counts);
        log::info!("{:18}: {}", "File counts", file_counts);
    }
}
