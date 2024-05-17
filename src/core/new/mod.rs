//! Implementation of the `new` subcommand.
use std::fs;
use std::path::PathBuf;
use std::{error::Error, path::Path};

use colored::Colorize;

use crate::cli::args::NewArgs;
use crate::helper::files::FileFinder;
use crate::helper::reads::{FastqReads, ReadAssignment, SampleNameFormat};
use crate::helper::utils;
use crate::types::SupportedFormats;

#[allow(dead_code)]
pub struct NewExecutor<'a> {
    dir: &'a Path,
    output: &'a Path,
    separator: Option<&'a str>,
    length: usize,
    re_sample: Option<&'a str>,
    is_recursive: bool,
    sample_name_format: SampleNameFormat,
}

impl<'a> NewExecutor<'a> {
    pub fn new(args: &'a NewArgs) -> Self {
        Self {
            dir: args.dir.as_path(),
            output: args.output.as_path(),
            separator: args.separator.as_deref(),
            length: args.length,
            re_sample: args.re_sample.as_deref(),
            is_recursive: args.recursive,
            sample_name_format: args
                .sample_name
                .parse::<SampleNameFormat>()
                .unwrap_or(SampleNameFormat::Simple),
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let spin = utils::init_spinner();
        spin.set_message("Finding files...");
        let format = SupportedFormats::Fastq;
        let files = FileFinder::new(&self.dir, &format).find(self.is_recursive)?;
        let records = self.assign_reads(&files);
        spin.set_message(format!(
            "Found {} samples of {} files",
            records.len(),
            files.len(),
        ));
        self.to_yaml(&records, files.len())?;
        spin.finish_with_message(format!("{} Finished creating a config file", "✔".green(),));

        log::info!("Output: {}", self.output.display());
        Ok(())
    }

    fn assign_reads(&self, files: &[PathBuf]) -> Vec<FastqReads> {
        ReadAssignment::new(&files, &self.sample_name_format).assign()
    }

    fn to_yaml(&self, records: &[FastqReads], file_counts: usize) -> Result<(), Box<dyn Error>> {
        let input_yaml = serde_yaml::to_string(&records)?;
        let config = NewConfig::new(
            self.dir,
            self.output,
            self.separator.unwrap_or(""),
            self.re_sample.unwrap_or(""),
            records.len(),
            file_counts,
            &input_yaml,
        );
        let yaml = serde_yaml::to_string(&config)?;
        fs::create_dir_all(&self.output)?;
        let output = self.output.join("config.yaml");
        let writer = std::fs::File::create(output)?;
        serde_yaml::to_writer(&writer, &yaml)?;
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NewConfig<'a> {
    input_dir: &'a Path,
    output: &'a Path,
    file_matching_strategy: &'a str,
    sample_name_format: &'a str,
    sample_counts: usize,
    file_counts: usize,
    input: &'a str,
}

impl<'a> NewConfig<'a> {
    pub fn new(
        input_dir: &'a Path,
        output: &'a Path,
        file_matching_strategy: &'a str,
        sample_name_format: &'a str,
        sample_counts: usize,
        file_counts: usize,
        input: &'a str,
    ) -> Self {
        Self {
            input_dir,
            output,
            file_matching_strategy,
            sample_name_format,
            sample_counts,
            file_counts,
            input,
        }
    }
}
