//! Generate checksum of a file.
//!
//! Supported checksum types:
//! - MD5
//! - SHA256
//!
//! We support MD5 for legacy purposes.
//! For better security, use SHA256.
use std::{
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::error::Error;
use md5::Md5;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Supported checksum types
pub enum ChecksumType {
    Md5,
    Sha256,
}

impl FromStr for ChecksumType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "md5" => Ok(ChecksumType::Md5),
            "sha256" => Ok(ChecksumType::Sha256),
            _ => Err(format!("Unknown checksum type: {}", s)),
        }
    }
}

impl Display for ChecksumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChecksumType::Md5 => write!(f, "md5"),
            ChecksumType::Sha256 => write!(f, "sha256"),
        }
    }
}

impl ChecksumType {
    /// Auto generate checksum based on the type
    /// Use if the type is not known at compile time
    pub fn generate(&self, file_path: &Path) -> Result<String, Error> {
        match self {
            ChecksumType::Md5 => self.md5(file_path),
            ChecksumType::Sha256 => self.sha256(file_path),
        }
    }

    /// Generate SHA256 hash of a file
    /// Use this method directly if you want to generate SHA256 hash
    /// Especially when doing a checksum in parallel
    pub fn sha256(&self, file_path: &Path) -> Result<String, Error> {
        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let mut hasher = Sha256::new();

        for byte in reader.bytes() {
            hasher.update([byte?]);
        }
        let value = hasher.finalize();
        Ok(format!("{:x}", value))
    }

    /// Generate MD5 hash of a file
    /// Use this method directly for parallel checksum generation
    /// It saves time by not checking the checksum type
    pub fn md5(&self, file_path: &Path) -> Result<String, Error> {
        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let mut hasher = Md5::new();

        for byte in reader.bytes() {
            hasher.update([byte?]);
        }
        let value = hasher.finalize();
        Ok(format!("{:x}", value))
    }
}

/// File metadata
/// Includes file path, size, and SHA256 hash
#[derive(Serialize)]
pub struct FileSha256 {
    /// Path to the file
    pub path: PathBuf,
    /// Size of the file in bytes
    pub size: u64,
    /// SHA256 hash of the file
    pub sha256: String,
}

impl FileSha256 {
    /// Initialize a new FileMeta instance
    pub fn new(path: PathBuf, size: u64, sha256: String) -> Self {
        Self { path, size, sha256 }
    }

    /// Convert file size to megabytes
    pub fn to_megabytes(&self) -> f64 {
        self.size as f64 / 1024.0 / 1024.0
    }
}
