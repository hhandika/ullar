//! Implementation of the `new` subcommand.
pub mod configs;
use std::path::PathBuf;
use std::{error::Error, path::Path};

use colored::Colorize;

use self::configs::{NewConfig, ReadMatching};
use crate::cli::args::NewArgs;
use crate::helper::files::FileFinder;
use crate::helper::reads::{FastqReads, ReadAssignment, SampleNameFormat};
use crate::helper::utils;
use crate::types::SupportedFormats;

pub struct NewExecutor<'a> {
    dir: &'a Path,
    output: &'a Path,
    extension: Option<&'a str>,
    separator: Option<char>,
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
            extension: args.extension.as_deref(),
            separator: args.separator,
            length: args.length,
            re_sample: args.re_sample.as_deref(),
            is_recursive: args.recursive,
            sample_name_format: args
                .sample_name
                .parse::<SampleNameFormat>()
                .expect("Invalid sample name format"),
        }
    }

    pub fn execute(&mut self) -> Result<(), Box<dyn Error>> {
        let spin = utils::init_spinner();
        spin.set_message("Finding files...");
        let format = SupportedFormats::Fastq;
        self.match_sample_name_format();
        let files = FileFinder::new(self.dir, &format).find(self.is_recursive)?;
        let records = self.assign_reads(&files);
        spin.set_message(format!(
            "Found {} samples of {} files",
            records.len(),
            files.len(),
        ));
        let output_path = self.write_config(&records, files.len())?;
        spin.finish_with_message(format!("{} Finished creating a config file", "✔".green(),));

        log::info!("Output: {}", output_path.display());
        Ok(())
    }

    fn match_sample_name_format(&mut self) {
        if let Some(regex) = self.re_sample {
            self.sample_name_format = SampleNameFormat::Custom(regex.to_string());
        }
    }

    fn assign_reads(&self, files: &[PathBuf]) -> Vec<FastqReads> {
        ReadAssignment::new(files, &self.sample_name_format).assign()
    }

    fn write_config(
        &self,
        records: &[FastqReads],
        file_counts: usize,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let data = serde_yaml::to_string(&records)?;
        let strategy: ReadMatching = self.get_read_matching_strategy();
        let extension = self.file_extension();
        let config = NewConfig::new(
            self.dir,
            &extension,
            records.len(),
            file_counts,
            strategy,
            &data,
        );
        let output_path = config.write_yaml(self.output)?;
        Ok(output_path)
    }

    fn file_extension(&self) -> String {
        if let Some(extension) = self.extension {
            extension.to_string()
        } else {
            String::from("default")
        }
    }

    fn get_read_matching_strategy(&self) -> ReadMatching {
        if let Some(separator) = self.separator {
            ReadMatching::character_split(separator, self.length)
        } else {
            ReadMatching::regex(self.sample_name_format.to_string())
        }
    }
}
