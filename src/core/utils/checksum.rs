//! Generate and print a SHA-256 hash of files

use std::{
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use clap::Error;
use colored::Colorize;
use comfy_table::Table;
use rayon::prelude::*;

use crate::{
    cli::args,
    helper::{
        checksum::{ChecksumType, FileSha256},
        files::{FileFinder, CSV_EXT},
        common,
    },
    types::SupportedFormats,
};

/// Hash a file using SHA256
/// Supports hashing multiple files and
/// is parallel by default
pub struct Checksum<'a> {
    /// Path to the file to hash
    pub files: &'a [PathBuf],
}

impl<'a> Checksum<'a> {
    /// Initialize a new FileHasher instance
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }

    /// Hash all files in the list in parallel
    /// Returns a vector of FileMetadata instances
    /// containing the file path, size, and SHA256 hash
    pub fn sha256(&self) -> Result<Vec<FileSha256>, Error> {
        let (tx, rx) = channel();

        self.files.par_iter().for_each_with(tx, |tx, file| {
            let meta = self
                .generate_meta_sha256(file)
                .expect("Failed to hash file");
            tx.send(meta).expect("Failed to send hash");
        });
        let file_hashes = rx.iter().collect::<Vec<FileSha256>>();
        Ok(file_hashes)
    }

    fn generate_meta_sha256(&self, file_path: &Path) -> Result<FileSha256, Error> {
        let size = file_path.metadata()?.len();
        let checksum = ChecksumType::Sha256;
        let sha256 = checksum.sha256(file_path)?;
        let meta = FileSha256::new(file_path.to_path_buf(), size, sha256);
        Ok(meta)
    }
}

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
        let spinner = common::init_spinner();
        spinner.set_message("Finding files...");
        let files = self.find_files()?;
        spinner.set_message(format!("Generating SHA256 from {} files...", files.len()));
        let hashes = Checksum::new(&files).sha256()?;
        spinner.finish_with_message(format!("{} Finished hashing files\n", "✔".green()));
        if self.is_stdout {
            self.write_stdout(&hashes);
        } else {
            self.write_csv(&hashes)?;
        }
        Ok(())
    }

    fn write_stdout(&self, hashes: &[FileSha256]) {
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

    fn write_csv(&self, hashes: &[FileSha256]) -> Result<(), Box<dyn std::error::Error>> {
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
        let files = FileFinder::new(self.dir, &self.format).find(self.is_recursive)?;
        Ok(files)
    }

    fn create_output_path(&self) -> PathBuf {
        self.output.with_extension(CSV_EXT)
    }
}
