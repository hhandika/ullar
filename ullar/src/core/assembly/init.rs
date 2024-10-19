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
    cli::commands::{assembly::AssemblyInitArgs, common::CommonInitArgs},
    core::clean::configs::{ReadConfig, ReadMatching},
    helper::{common, files::FileFinder},
    types::{
        reads::{FastqReads, ReadAssignment, SampleNameFormat},
        SupportedFormats,
    },
};
pub struct AssemblyInit<'a> {
    input_dir: &'a Path,
    common: &'a CommonInitArgs,
    sample_name_format: SampleNameFormat,
}

impl<'a> AssemblyInit<'a> {
    pub fn from_arg(args: &'a AssemblyInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            common: &args.common,
            sample_name_format: args
                .common
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
        let files = FileFinder::new(self.input_dir, &format).find(self.common.recursive)?;
        let file_count = files.len();
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

        let config_path = self.write_config(records, file_count)?;
        spin.finish_with_message(format!(
            "Config file generated at: {}",
            config_path.display()
        ));
        Ok(())
    }

    fn match_sample_name_format(&mut self) {
        if let Some(regex) = &self.common.re_sample {
            self.sample_name_format = SampleNameFormat::Custom(regex.to_string());
        }
    }

    fn assign_reads(&self, files: &[PathBuf]) -> Vec<FastqReads> {
        ReadAssignment::new(files, &self.sample_name_format).assign()
    }

    fn get_read_matching_strategy(&self) -> ReadMatching {
        if let Some(separator) = self.common.separator {
            ReadMatching::character_split(separator, self.common.length)
        } else {
            ReadMatching::regex(self.sample_name_format.to_string())
        }
    }

    fn write_config(
        &self,
        records: Vec<FastqReads>,
        file_counts: usize,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let strategy = self.get_read_matching_strategy();
        let mut config = ReadConfig::new(
            self.input_dir,
            self.file_extension(),
            records.len(),
            file_counts,
            strategy,
            records.to_vec(),
        );
        let output_path = config.to_yaml()?;
        Ok(output_path)
    }

    fn file_extension(&self) -> String {
        if let Some(extension) = &self.common.extension {
            extension.to_string()
        } else {
            String::from("default")
        }
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("Directory: {:?}", self.input_dir);
        log::info!("{:18}: {}\n", "Sample name format", self.common.sample_name);
    }
}
