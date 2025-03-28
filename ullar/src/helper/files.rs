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
    helper::regex::{
        CONTIG_REGEX, FASTA_REGEX, FASTQ_REGEX, NEXUS_REGEX, PHYLIP_REGEX, PLAIN_TEXT_REGEX,
    },
    re_match,
    types::SupportedFormats,
};

use super::checksum::ChecksumType;

pub const CSV_EXT: &str = "csv";

#[macro_export]
macro_rules! get_file_stem {
    ($self:ident, $path:ident) => {
        $self
            .$path
            .file_stem()
            .unwrap_or_else(|| $self.$path.file_name().expect("Failed to get file name"))
            .to_string_lossy()
            .to_string()
    };
}

/// Path checking utility
pub struct PathCheck<'a> {
    /// Path to check
    path: &'a Path,
    /// Is the path a directory
    is_dir: bool,
    /// Force overwrite of existing files
    force: bool,
}

impl<'a> PathCheck<'a> {
    /// Initialize a new PathCheck instance
    pub fn new(path: &'a Path) -> Self {
        Self {
            path,
            is_dir: false,
            force: false,
        }
    }

    pub fn with_force_overwrite(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn is_dir(mut self) -> Self {
        self.is_dir = true;
        self
    }

    /// Check if the path exists and prompt the user to delete it
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
            if self.force {
                self.delete();
            } else {
                self.prompt_users(&message);
            }
        }
    }

    fn prompt_users(&self, message: &str) {
        let selection = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .interact();
        match selection {
            Ok(true) => self.delete(),
            Ok(false) => {
                log::info!(
                    "\nCancel deleting the {} directory.\n\
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

    fn delete(&self) {
        if self.is_dir {
            self.delete_dir();
        } else {
            self.delete_file();
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
        let msg = format!(
            "\nFile {} has been removed.\n",
            self.path.display().to_string().red()
        );
        log::warn!("{}", msg);
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
            SupportedFormats::Contigs => re_match!(CONTIG_REGEX, path),
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

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let file = fs::metadata(path.as_ref()).unwrap_or_else(|_| {
            panic!(
                "Failed to get metadata for {}",
                path.as_ref().display().to_string().red()
            )
        });
        let file_name = path
            .as_ref()
            .file_name()
            .expect("Failed to get file name")
            .to_str()
            .expect("Failed to convert file name to string")
            .to_string();
        let parent_dir = path
            .as_ref()
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();
        let file_size = Size::from_bytes(file.len()).to_string();
        let checksum = ChecksumType::Sha256;
        let sha256 = checksum
            .generate(path.as_ref())
            .expect("Failed to generate SHA256 hash");
        Self {
            file_name,
            parent_dir,
            file_size,
            sha256,
        }
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
    fn test_re_contig() {
        let path = Path::new("contigs.fasta");
        let format = SupportedFormats::Contigs;
        let finder = FileFinder::new(path, &format);
        assert!(finder.is_matching_file(path));
    }

    #[test]
    fn test_read_finder() {
        let dir = Path::new("tests/data/reads");
        let format = SupportedFormats::Fastq;
        let finder = FileFinder::new(dir, &format);
        let files = finder.find_files().expect("Failed to find files");
        assert_eq!(files.len(), 4);
    }
}
