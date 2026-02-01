//! Automatic file finder for all supported file types

use std::{
    fs::{self},
    io::Error,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;
use walkdir::WalkDir;

use crate::{files::regex::FASTQ_REGEX, re_match};

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

/// Find all raw read files in the specified directory
pub struct FileFinder<'a> {
    /// Directory to search for raw read files
    pub dir: &'a Path,
}

impl<'a> FileFinder<'a> {
    /// Initialize a new ReadFinder instance
    pub fn new(dir: &'a Path) -> Self {
        FileFinder { dir }
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
            .filter(|e| re_match!(FASTQ_REGEX, e))
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
        re_match!(FASTQ_REGEX, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_match_fastq() {
        let path = Path::new("test.fastq");
        let finder = FileFinder::new(path);
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
            let finder = FileFinder::new(path);
            assert!(finder.is_matching_file(path));
        }
    }
}
