//! Automatic file finder for all supported file types

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;
use walkdir::WalkDir;

use crate::{
    helper::regex::{FASTA_REGEX, FASTQ_REGEX, NEXUS_REGEX, PHYLIP_REGEX, PLAIN_TEXT_REGEX},
    re_match,
    types::SupportedFormats,
};

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
    /// Find all files in the directory
    pub fn find_files(&self) -> Result<Vec<PathBuf>, Error> {
        let files = fs::read_dir(self.dir)?
            .map(|entry| entry.map(|e| e.path()))
            .filter_map(|e| e.ok())
            .filter(|e| e.is_file())
            .filter(|e| self.is_matching_file(e))
            .collect::<Vec<PathBuf>>();

        Ok(files)
    }

    /// Find all files in a directory and its subdirectories
    pub fn find_files_recursive(&self) -> Result<Vec<PathBuf>, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_match_fastq() {
        let path = Path::new("test.fastq");
        let format = SupportedFormats::Fastq;
        let finder = FileFinder::new(path, &format);
        assert_eq!(finder.is_matching_file(path), true);
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
            assert_eq!(finder.is_matching_file(path), true);
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
            assert_eq!(finder.is_matching_file(path), true);
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
