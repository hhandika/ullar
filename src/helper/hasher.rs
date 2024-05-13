use std::{
    collections::BTreeMap,
    fs,
    io::{Error, Read},
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use sha2::{Digest, Sha256};

/// Hash a file using SHA256
/// Supports hashing multiple files and
/// is parallel by default
pub struct Hasher<'a> {
    /// Path to the file to hash
    pub files: &'a [PathBuf],
}

impl<'a> Hasher<'a> {
    /// Initialize a new FileHasher instance
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }
    /// Hash all files in the list in parallel
    /// Returns a HashMap of file paths and their corresponding SHA256 hash
    pub fn sha256(&self) -> Result<BTreeMap<PathBuf, String>, Error> {
        let (tx, rx) = channel();

        self.files.par_iter().for_each_with(tx, |tx, file| {
            let hash = self.hash(file).expect("Failed to hash file");
            tx.send((file.clone(), hash)).expect("Failed to send hash");
        });
        let file_hashes = rx.iter().collect::<BTreeMap<PathBuf, String>>();
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
