//! Implementation of the `new` subcommand.
use std::{error::Error, path::Path};

use comfy_table::Table;

use crate::helper::reads::{ReadAssignment, SampleNameFormat};
use crate::{cli::args::NewArgs, helper::files::FileFinder, types::SupportedFormats};

#[allow(dead_code)]
pub struct NewExecutor<'a> {
    dir: &'a Path,
    output: &'a Path,
    separator: Option<&'a str>,
    length: usize,
    re_file: Option<&'a str>,
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
            re_file: args.extension.as_deref(),
            re_sample: args.re_sample.as_deref(),
            is_recursive: args.recursive,
            sample_name_format: args
                .sample_name
                .parse::<SampleNameFormat>()
                .unwrap_or(SampleNameFormat::Simple),
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let file_format = SupportedFormats::Fastq;
        let files = FileFinder::new(self.dir, &file_format)
            .find_files()
            .expect("Failed to find files");

        let reads = ReadAssignment::new(&files, &self.sample_name_format).assign();

        let mut table = Table::new();
        table.set_header(vec!["Sample Name", "Read1", "Read2", "Singletons"]);

        for (sample_name, reads) in reads {
            table.add_row(vec![
                sample_name,
                reads.read_1.to_string(),
                reads.read_2.unwrap_or_default(),
                reads.singletons.unwrap_or_default(),
            ]);
        }

        println!("{}", table);

        Ok(())
    }
}
