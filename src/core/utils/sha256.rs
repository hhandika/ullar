//! Generate and print a SHA-256 hash of files

use std::path::{Path, PathBuf};

use clap::Error;
use colored::Colorize;
use comfy_table::Table;

use crate::{
    cli::args,
    helper::{
        files::FileFinder,
        hasher::{FileMetadata, Hasher},
        utils,
    },
    types::SupportedFormats,
};

const CSV_EXT: &str = "csv";

/// Execute sha256 generation command
pub struct Sha256Executor<'a> {
    dir: &'a Path,
    output: &'a Path,
    format: SupportedFormats,
    is_stdout: bool,
    is_recursive: bool,
}

impl<'a> Sha256Executor<'a> {
    pub fn new(args: &'a args::Sha256Args) -> Self {
        Self {
            dir: args.dir.as_path(),
            output: args.output.as_path(),
            format: args
                .format
                .parse::<SupportedFormats>()
                .expect("Failed to parse format"),
            is_stdout: args.stdout,
            is_recursive: args.recursive,
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let spinner = utils::create_spinner();
        spinner.set_message("Finding files...");
        let files = self.find_files()?;
        spinner.set_message("Hashing files...");
        let hashes = Hasher::new(&files).sha256()?;
        spinner.finish_with_message(format!("{} Finished hashing files\n", "✔".green()));
        if self.is_stdout {
            self.write_stdout(&hashes);
        } else {
            self.write_csv(&hashes)?;
        }
        Ok(())
    }

    fn write_stdout(&self, hashes: &[FileMetadata]) {
        let mut table = Table::new();
        table.set_header(vec!["File", "Size (Mb)", "SHA256"]);
        for hash in hashes {
            table.add_row(vec![
                hash.path
                    .file_name()
                    .expect("Failed to get file name")
                    .to_string_lossy()
                    .to_string(),
                format!("{:.2}", hash.to_megabytes()),
                hash.sha256.to_string(),
            ]);
        }
        println!("{table}");
    }

    fn write_csv(&self, hashes: &[FileMetadata]) -> Result<(), Box<dyn std::error::Error>> {
        let output = self.create_output_path();
        let mut writer = csv::Writer::from_path(&output)?;
        for hash in hashes {
            writer.serialize(hash)?;
        }
        writer.flush()?;
        log::info!("SHA256 hashes written to: {}", output.display());
        Ok(())
    }

    fn find_files(&self) -> Result<Vec<PathBuf>, Error> {
        let finder = FileFinder::new(&self.dir, &self.format);
        let files = if self.is_recursive {
            finder.find_files_recursive()?
        } else {
            finder.find_files()?
        };
        Ok(files)
    }

    fn create_output_path(&self) -> PathBuf {
        self.output.with_extension(CSV_EXT)
    }
}
