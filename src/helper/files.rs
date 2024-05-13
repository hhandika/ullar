//! Automatic file finder for all supported file types

use std::{
    collections::HashMap,
    fs,
    io::{Error, Read},
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use sha2::{Digest, Sha256};

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

/// Hash a file using SHA256
pub struct FileHasher<'a> {
    /// Path to the file to hash
    pub files: &'a [PathBuf],
}

impl<'a> FileHasher<'a> {
    /// Initialize a new FileHasher instance
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }
    /// Hash all files in the list in parallel
    /// Returns a HashMap of file paths and their corresponding SHA256 hash
    pub fn sha256(&self) -> Result<HashMap<PathBuf, String>, Error> {
        let (tx, rx) = channel();

        self.files.par_iter().for_each_with(tx, |tx, file| {
            let hash = self.hash(file).expect("Failed to hash file");
            tx.send((file.clone(), hash)).expect("Failed to send hash");
        });
        let file_hashes = rx.iter().collect::<HashMap<PathBuf, String>>();
        Ok(file_hashes)
    }

    pub fn hash(&self, file_path: &Path) -> Result<String, Error> {
        let file = fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let mut hasher = Sha256::new();

        for byte in reader.bytes() {
            hasher.update(&[byte?]);
        }
        let value = hasher.finalize();

        Ok(format!("{:x}", value))
    }
}

/// Create SHA256 hash of a file

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

// "sample1_R1.fastq",
//         "sample1_R2.fastq",
//         "sample1_singleton.fastq",
//         "sample2_1.fastq.gz",
//         "sample2_2.fastq.gz",
//         "control3_read1.fastq.bz2",
//         "control3_read2.fastq.bz2",
//         "control3_singleton.fastq",
//         "sample3_R1.fastq.xz",
//         "sample3_R2.fastq.xz",

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
