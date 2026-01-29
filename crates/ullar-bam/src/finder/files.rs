//! Find BAM files in a directory for batch processing.
//!

use std::path::Path;
use walkdir::WalkDir;

use crate::types::BamFormat;

pub struct BamFileFinder<'a> {
    pub dir: &'a Path,
    pub recursive: bool,
    pub format: BamFormat,
}

impl<'a> BamFileFinder<'a> {
    pub fn new(dir: &'a Path, recursive: bool, format: BamFormat) -> Self {
        BamFileFinder {
            dir,
            recursive,
            format,
        }
    }

    pub fn find(&self) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        if self.recursive {
            self.find_file_recursive()
        } else {
            self.find_files()
        }
    }

    fn find_files(&self) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        let files = std::fs::read_dir(self.dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter(|entry| self.matches_format(&entry.path()))
            .map(|entry| entry.path())
            .collect();
        Ok(files)
    }

    fn find_file_recursive(&self) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        let files = WalkDir::new(self.dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| self.matches_format(entry.path()))
            .map(|entry| entry.path().to_path_buf())
            .collect();
        Ok(files)
    }

    fn matches_format(&self, path: &Path) -> bool {
        if let Some(bam_format) = BamFormat::from_path(path) {
            return bam_format == self.format;
        }
        false
    }
}
