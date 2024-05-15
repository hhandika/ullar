use std::{
    fs,
    io::{Error, Read},
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use serde::Serialize;
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
    /// Returns a vector of FileMetadata instances
    /// containing the file path, size, and SHA256 hash
    pub fn sha256(&self) -> Result<Vec<FileMetadata>, Error> {
        let (tx, rx) = channel();

        self.files.par_iter().for_each_with(tx, |tx, file| {
            let meta = self
                .generate_meta_sha256(file)
                .expect("Failed to hash file");
            tx.send(meta).expect("Failed to send hash");
        });
        let file_hashes = rx.iter().collect::<Vec<FileMetadata>>();
        Ok(file_hashes)
    }

    /// Generate SHA256 hash of a file
    /// Returns the hash as a string
    pub fn generate_sha256(&self, file_path: &Path) -> Result<String, Error> {
        let file = fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let mut hasher = Sha256::new();

        for byte in reader.bytes() {
            hasher.update(&[byte?]);
        }
        let value = hasher.finalize();
        Ok(format!("{:x}", value))
    }

    fn generate_meta_sha256(&self, file_path: &Path) -> Result<FileMetadata, Error> {
        let size = file_path.metadata()?.len();
        let sha256 = self.generate_sha256(file_path)?;
        let meta = FileMetadata::new(file_path.to_path_buf(), size, sha256);
        Ok(meta)
    }
}

/// File metadata
/// Includes file path, size, and SHA256 hash
#[derive(Serialize)]
pub struct FileMetadata {
    /// Path to the file
    pub path: PathBuf,
    /// Size of the file in bytes
    pub size: u64,
    /// SHA256 hash of the file
    pub sha256: String,
}

impl FileMetadata {
    /// Initialize a new FileMeta instance
    pub fn new(path: PathBuf, size: u64, sha256: String) -> Self {
        Self { path, size, sha256 }
    }

    /// Convert file size to megabytes
    pub fn to_megabytes(&self) -> f64 {
        self.size as f64 / 1024.0 / 1024.0
    }
}
