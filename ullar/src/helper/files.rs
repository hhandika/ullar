//! Automatic file finder for all supported file types

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use size::Size;
use walkdir::WalkDir;

use crate::{
    helper::regex::{FASTA_REGEX, FASTQ_REGEX, NEXUS_REGEX, PHYLIP_REGEX, PLAIN_TEXT_REGEX},
    re_match,
    types::SupportedFormats,
};

use super::checksum::ChecksumType;

pub const CSV_EXT: &str = "csv";

pub struct PathCheck<'a> {
    path: &'a Path,
    is_dir: bool,
}

impl<'a> PathCheck<'a> {
    pub fn new(path: &'a Path, is_dir: bool) -> Self {
        Self { path, is_dir }
    }

    pub fn prompt_exists(&self, dry_run: bool) {
        let message = format!(
            "Path {} already exists. Do you want to delete it?",
            self.path.display().to_string().red()
        );

        if dry_run && self.path.exists() {
            log::warn!(
                "\nPath {} already exists. \
                Skipping deletion for dry run...\n",
                self.path.display().to_string().red()
            );
            return;
        }

        if self.path.exists() {
            self.prompt_users(&message);
        }
    }

    fn prompt_users(&self, message: &str) {
        let selection = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .interact();
        match selection {
            Ok(true) => {
                if self.is_dir {
                    self.delete_dir();
                } else {
                    self.delete_file();
                }
            }
            Ok(false) => {
                log::info!(
                    "\nAbort deleting the {} directory.\n\
                    Rename the directory manually to proceed.\n\
                    Exiting program...\n",
                    self.path.display().to_string().red()
                );
                log::info!("");
                std::process::exit(0);
            }
            Err(_) => {
                log::error!("\nFailed to get user input. Exiting program...\n");
                log::info!("");
                std::process::exit(1);
            }
        }
    }

    fn delete_dir(&self) {
        fs::remove_dir_all(self.path).expect("Failed to remove directory");
        let msg = format!(
            "\nDirectory {} has been removed.\n",
            self.path.display().to_string().red()
        );
        log::warn!("{}", msg);
    }

    fn delete_file(&self) {
        fs::remove_file(self.path).expect("Failed to remove file");
        let msg = format!("\nFile {} has been removed.\n", self.path.display());
        log::warn!("{}", msg.red());
    }
}

/// Find all raw read files in the specified directory
pub struct FileFinder<'a> {
    /// Directory to search for raw read files
    pub dir: &'a Path,
    /// File format to search for
    pub format: &'a SupportedFormats,
}

impl<'a> FileFinder<'a> {
    /// Initialize a new ReadFinder instance
    pub fn new(dir: &'a Path, format: &'a SupportedFormats) -> Self {
        FileFinder { dir, format }
    }

    /// Find files in the directory
    /// If is_recursive is true, find files in the directory and its subdirectories
    /// Otherwise, find files in the directory only
    pub fn find(&self, is_recursive: bool) -> Result<Vec<PathBuf>, Error> {
        if is_recursive {
            self.find_files_recursive()
        } else {
            self.find_files()
        }
    }

    fn find_files(&self) -> Result<Vec<PathBuf>, Error> {
        let files = fs::read_dir(self.dir)?
            .map(|entry| entry.map(|e| e.path()))
            .filter_map(|e| e.ok())
            .filter(|e| e.is_file())
            .filter(|e| self.is_matching_file(e))
            .collect::<Vec<PathBuf>>();

        Ok(files)
    }

    fn find_files_recursive(&self) -> Result<Vec<PathBuf>, Error> {
        let files = WalkDir::new(self.dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .filter(|e| e.is_file())
            .filter(|e| self.is_matching_file(e))
            .collect::<Vec<PathBuf>>();

        Ok(files)
    }

    fn is_matching_file(&self, path: &Path) -> bool {
        match self.format {
            SupportedFormats::Fastq => re_match!(FASTQ_REGEX, path),
            SupportedFormats::Fasta => re_match!(FASTA_REGEX, path),
            SupportedFormats::Nexus => re_match!(NEXUS_REGEX, path),
            SupportedFormats::Phylip => re_match!(PHYLIP_REGEX, path),
            SupportedFormats::PlainText => re_match!(PLAIN_TEXT_REGEX, path),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct FileMetadata {
    pub file_name: String,
    pub parent_dir: PathBuf,
    pub file_size: String,
    pub sha256: String,
}

impl FileMetadata {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            parent_dir: PathBuf::new(),
            file_size: String::new(),
            sha256: String::new(),
        }
    }

    pub fn get(&mut self, path: &Path) {
        let file = fs::metadata(path).expect("Failed to get file metadata");
        self.file_name = path
            .file_name()
            .expect("Failed to get file name")
            .to_str()
            .expect("Failed to convert file name to string")
            .to_string();
        self.parent_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        self.file_size = Size::from_bytes(file.len()).to_string();
        let checksum = ChecksumType::Sha256;
        self.sha256 = checksum
            .generate(path)
            .expect("Failed to generate SHA256 hash");
    }

    pub fn canonicalize(&self) -> PathBuf {
        self.parent_dir
            .join(&self.file_name)
            .canonicalize()
            .expect("Failed to canonicalize path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_match_fastq() {
        let path = Path::new("test.fastq");
        let format = SupportedFormats::Fastq;
        let finder = FileFinder::new(path, &format);
        assert!(finder.is_matching_file(path));
    }

    #[test]
    fn test_re_match_all_fastq() {
        let paths: Vec<&str> = vec![
            "sample1_R1.fastq",
            "sample1_R2.fastq",
            "sample1_singleton.fastq",
            "sample2_1.fastq.gz",
            "sample2_2.fastq.gz",
            "control3_read1.fastq.bz2",
            "control3_read2.fastq.bz2",
            "control3_singleton.fastq",
            "sample3_R1.fastq.xz",
            "sample3_R2.fastq.xz",
        ];
        for path in paths {
            let path = Path::new(path);
            let format = SupportedFormats::Fastq;
            let finder = FileFinder::new(path, &format);
            assert!(finder.is_matching_file(path));
        }
    }

    #[test]
    fn test_re_match_all_fasta() {
        let paths: Vec<&str> = vec![
            "sample1.fasta",
            "sample2.fa",
            "sample3.fna",
            "sample4.fsa",
            "sample5.fas",
        ];
        for path in paths {
            let path = Path::new(path);
            let format = SupportedFormats::Fasta;
            let finder = FileFinder::new(path, &format);
            assert!(finder.is_matching_file(path));
        }
    }

    #[test]
    fn test_read_finder() {
        let dir = Path::new("tests/reads");
        let format = SupportedFormats::Fastq;
        let finder = FileFinder::new(dir, &format);
        let files = finder.find_files().expect("Failed to find files");
        assert_eq!(files.len(), 4);
    }
}
