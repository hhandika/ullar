//! A scan command executor
use std::path::{Path, PathBuf};

use colored::Colorize;
use comfy_table::Table;

use crate::{
    cli::args::ReadScanArgs,
    helper::{
        files::{FileFinder, CSV_EXT},
        reads::{FastqReads, ReadAssignment, SampleNameFormat},
        utils,
    },
    types::SupportedFormats,
};

pub struct ReadScanner<'a> {
    dir: &'a Path,
    output: &'a Path,
    is_stdout: bool,
    is_recursive: bool,
    sample_name_format: SampleNameFormat,
}

impl<'a> ReadScanner<'a> {
    pub fn new(args: &'a ReadScanArgs) -> Self {
        Self {
            dir: args.dir.as_path(),
            output: args.output.as_path(),
            is_stdout: args.stdout,
            is_recursive: args.recursive,
            sample_name_format: args
                .sample_name
                .parse::<SampleNameFormat>()
                .unwrap_or(SampleNameFormat::Simple),
        }
    }

    pub fn scan(&self) -> Result<(), Box<dyn std::error::Error>> {
        let spinner = utils::init_spinner();
        spinner.set_message("Scanning FASTQ reads...");
        let format = SupportedFormats::Fastq;
        let files = self.find_files(&format);
        let records = ReadAssignment::new(&files, &self.sample_name_format).assign();

        spinner.finish_with_message(format!("{} Finished scanning files\n", "✔".green()));

        if self.is_stdout {
            self.write_stdout(&records);
        } else {
            self.write_csv(&records).expect("Failed to write CSV");
        }

        Ok(())
    }

    fn write_stdout(&self, records: &[FastqReads]) {
        let mut table = Table::new();
        table.set_header(vec!["Sample Name", "Read1", "Read2", "Singletons"]);

        for reads in records {
            table.add_row(vec![
                &reads.sample_name,
                &reads
                    .read_1
                    .as_ref()
                    .map_or(String::new(), |r| r.file_name.to_string()),
                &reads
                    .read_2
                    .as_ref()
                    .map_or(String::new(), |r| r.file_name.to_string()),
                &reads
                    .singletons
                    .as_ref()
                    .map_or(String::new(), |r| r.file_name.to_string()),
            ]);
        }
        println!("{table}");
    }

    fn write_csv(&self, records: &[FastqReads]) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.output.with_extension(CSV_EXT);
        let mut writer = csv::Writer::from_path(&output_path)?;
        writer.write_record(&["sample_name", "read1", "read2", "singletons"])?;
        for reads in records {
            writer.serialize((
                &reads.sample_name,
                &reads.read_1,
                &reads.read_2,
                &reads.singletons,
            ))?;
        }
        writer.flush()?;
        log::info!("FASTQ reads written to: {}", output_path.display());
        Ok(())
    }

    fn find_files(&self, format: &SupportedFormats) -> Vec<PathBuf> {
        FileFinder::new(self.dir, format)
            .find(self.is_recursive)
            .expect("Failed to find files")
    }
}
