//! Automatic file finder for all supported file types

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::types::SupportedFormats;

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
        re_match(path, self.format)
    }
}

/// Create SHA256 hash of a file

// Match fastq and fastq.gz files
// It does all matching for now.
// Later, we will separate the matching for read 1, 2 and single end reads
fn re_match(path: &Path, format: &SupportedFormats) -> bool {
    let pattern = format.to_regex();
    let re = regex::Regex::new(pattern).expect("Failed to compile regex");
    let file_name = path.file_name().expect("Failed to get file name");
    re.is_match(
        file_name
            .to_str()
            .expect("Failed to convert file name to string"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_match_fastq() {
        let path = Path::new("test.fastq");
        let format = SupportedFormats::Fastq;
        assert_eq!(re_match(path, &format), true);
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
            assert_eq!(re_match(path, &format), true);
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
            assert_eq!(re_match(path, &format), true);
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
