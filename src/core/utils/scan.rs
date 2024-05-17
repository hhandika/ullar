//! A scan command executor
use std::path::{Path, PathBuf};

use colored::Colorize;
use comfy_table::Table;
use segul::helper::utils;

use crate::{
    cli::args::ScanArgs,
    helper::{
        files::FileFinder,
        reads::{ReadAssignment, SampleNameFormat},
    },
    types::{SupportedDataTypes, SupportedFormats},
};

#[allow(dead_code)]
pub struct ScanExecutor<'a> {
    dir: &'a Path,
    output: &'a Path,
    datatype: SupportedDataTypes,
    is_stdout: bool,
    is_recursive: bool,
    sample_name_format: SampleNameFormat,
}

impl<'a> ScanExecutor<'a> {
    pub fn new(args: &'a ScanArgs) -> Self {
        Self {
            dir: args.dir.as_path(),
            output: args.output.as_path(),
            datatype: args
                .datatype
                .parse::<SupportedDataTypes>()
                .expect("Failed to parse format"),
            is_stdout: args.stdout,
            is_recursive: args.recursive,
            sample_name_format: args
                .sample_name
                .parse::<SampleNameFormat>()
                .unwrap_or(SampleNameFormat::Simple),
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        if SupportedDataTypes::Alignment == self.datatype {
            let format: Vec<SupportedFormats> =
                vec![SupportedFormats::Nexus, SupportedFormats::Phylip];
            let mut files = Vec::new();
            for f in format {
                files.append(&mut self.find_files(&f));
            }
        } else {
            self.scan_files();
        };

        Ok(())
    }

    fn scan_files(&self) {
        match self.datatype {
            SupportedDataTypes::RawReads => self.scan_reads(),
            SupportedDataTypes::Contigs => unimplemented!(),
            SupportedDataTypes::Tree => unimplemented!(),
            _ => unreachable!("Unsupported data type"),
        }
    }

    fn scan_reads(&self) {
        let spinner = utils::set_spinner();
        spinner.set_message("Scanning FASTQ reads...");
        let format = SupportedFormats::Fastq;
        let files = self.find_files(&format);
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

        spinner.finish_with_message(format!("{} Finished scanning files\n", "✔".green()));

        println!("{}", table);
    }

    fn find_files(&self, format: &SupportedFormats) -> Vec<PathBuf> {
        FileFinder::new(self.dir, format)
            .find(self.is_recursive)
            .expect("Failed to find files")
    }
}
