//! Automatic file finder for all supported file types

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};

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

use super::hasher::generate_sha256;

pub const CSV_EXT: &str = "csv";

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
    pub mime_type: String,
    pub sha256: String,
}

impl FileMetadata {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            parent_dir: PathBuf::new(),
            file_size: String::new(),
            mime_type: String::new(),
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
        self.mime_type = self.get_mime_type(path);
        self.sha256 = generate_sha256(path).expect("Failed to generate SHA256 hash");
    }

    fn get_mime_type(&self, path: &Path) -> String {
        let mime = infer::get_from_path(path).expect("Failed to get MIME type");
        match mime {
            Some(mime) => mime.mime_type().to_string(),
            None => String::from("unknown"),
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
