//! Implementation of the `new` subcommand.

use std::path::PathBuf;
use std::{error::Error, path::Path};

use colored::Colorize;

use crate::cli::commands::clean::ReadCleaningInitArgs;
use crate::cli::commands::common::{CommonInitArgs, GenomicReadsInitArgs};
use crate::core::clean::ReadCleaning;
use crate::helper::common;
use crate::helper::fastq::{FastqInput, ReadAssignmentStrategy};
use crate::helper::files::FileFinder;
use crate::types::reads::{FastqReads, ReadAssignment, SampleNameFormat};
use crate::types::SupportedFormats;

use super::configs::CleanReadConfig;

pub struct ReadCleaningInit<'a> {
    input_dir: &'a Path,
    common: &'a CommonInitArgs,
    reads: &'a GenomicReadsInitArgs,
    sample_name_format: SampleNameFormat,
}

impl<'a> ReadCleaningInit<'a> {
    pub fn from_arg(args: &'a ReadCleaningInitArgs) -> Self {
        Self {
            input_dir: args.dir.as_path(),
            common: &args.common,
            reads: &args.reads,
            sample_name_format: args
                .reads
                .sample_name
                .parse::<SampleNameFormat>()
                .expect("Invalid sample name format"),
        }
    }

    pub fn init(&mut self) {
        self.log_input();
        let spin = common::init_spinner();
        spin.set_message("Finding files...");
        let format = SupportedFormats::Fastq;
        self.match_sample_name_format();
        let files = FileFinder::new(self.input_dir, &format)
            .find(self.reads.recursive)
            .expect(
                "Failed to find files. \
            Check if the directory exists and you have permission to access it.",
            );
        let file_count = files.len();
        if files.is_empty() {
            spin.finish_with_message(format!(
                "{} No files found in {}. \
                Try using the --recursive flag if files are in subdirectories.",
                "✖".red(),
                self.input_dir.display()
            ));
            log::error!("Try using the --recursive flag if files are in subdirectories.");
        }
        spin.set_message(format!(
            "Found {} files. Assigning reads and generating hash for matching files...",
            file_count
        ));
        let records = self.assign_reads(&files);
        let record_count = records.len();
        spin.set_message(format!(
            "Found {} samples of {} files. Writing config file...",
            record_count, file_count
        ));
        let config = self.write_config(records, files.len());
        match config {
            Ok(path) => {
                spin.finish_with_message(format!(
                    "{} Finished creating a config file\n",
                    "✔".green()
                ));
                self.log_output(&path, record_count, file_count);
                if self.common.autorun {
                    let footer = common::PrettyHeader::new();
                    footer.get_section_footer();
                    self.autorun_pipeline(&path);
                }
            }
            Err(e) => {
                spin.finish_with_message(format!(
                    "{} Failed to write config file: {}",
                    "✖".red(),
                    e
                ));
                log::error!("Failed to write config file: {}", e);
                return;
            }
        };
    }

    fn autorun_pipeline(&self, config_path: &Path) {
        let header = "Starting read cleaning pipeline...".to_string();
        log::info!("{}", header.cyan());
        log::info!("");
        let runner = ReadCleaning::from_config_path(config_path);
        runner.clean();
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
        let mut config = CleanReadConfig::new(input_summary, records.to_vec());
        let output_path = config.to_toml(self.common.override_args.as_deref())?;
        Ok(output_path)
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.input_dir.display());
        log::info!("{:18}: {}\n", "Sample name format", self.sample_name_format);
    }

    fn log_output(&self, output_path: &Path, record_counts: usize, file_counts: usize) {
        let config_filename = output_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        log::info!("{}", "\nOutput".cyan());
        log::info!(
            "{:18}: {}",
            "Directory",
            output_path
                .parent()
                .expect("Failed to parse parent directory")
                .display()
        );
        log::info!("{:18}: {}", "Config file", config_filename);
        log::info!("{:18}: {}", "Sample counts", record_counts);
        log::info!("{:18}: {}", "File counts", file_counts);
    }
}
