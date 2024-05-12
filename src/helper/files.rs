//! Automatic file finder for all supported file types

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};

use super::types::RawReadFormat;

/// Find all raw read files in the specified directory
pub struct ReadFinder<'a> {
    /// Directory to search for raw read files
    pub dir: &'a Path,
    /// File format to search for
    pub format: &'a RawReadFormat,
}

impl<'a> ReadFinder<'a> {
    /// Initialize a new ReadFinder instance
    pub fn new(dir: &'a Path, format: &'a RawReadFormat) -> Self {
        ReadFinder { dir, format }
    }
    /// Create a new ReadFinder instance
    pub fn find_files(&self) -> Result<Vec<PathBuf>, Error> {
        let files = fs::read_dir(self.dir)?
            .map(|entry| entry.map(|e| e.path()))
            .filter_map(|e| e.ok())
            .filter(|e| e.is_file())
            .filter(|e| re_match_fastq(e))
            .collect::<Vec<_>>();

        Ok(files)
    }
}

// Match fastq and fastq.gz files
// It does all matching for now.
// Later, we will separate the matching for read 1, 2 and single end reads
fn re_match_fastq(path: &Path) -> bool {
    let pattern = r"(?i)(.fq|.fastq)(?:.*)";
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
        assert_eq!(re_match_fastq(path), true);
    }

    #[test]
    fn test_read_finder() {
        let dir = Path::new("tests/reads");
        let format = RawReadFormat::Auto;
        let finder = ReadFinder::new(dir, &format);
        let files = finder.find_files().expect("Failed to find files");
        assert_eq!(files.len(), 4);
    }
}
