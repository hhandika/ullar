//! Match file path to FASTQ reads

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use colored::Colorize;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    helper::regex::{DESCRIPTIVE_NAME_REGEX, READ1_REGEX, READ2_REGEX, SIMPLE_NAME_REGEX},
    re_capture, re_capture_dynamic, re_capture_lazy, re_match,
};

use super::{checksum::ChecksumType, files::FileMetadata};

#[macro_export]
macro_rules! check_read1_exists {
    ($self: ident, $read1: ident) => {
        if !$read1.exists() {
            let msg = format!(
                "\nRead 1 file not found for {}. Skipping it!\n",
                $self.sample.sample_name
            );
            log::error!("{}", msg.red());
            return Err("Read 1 file not found".into());
        }
    };
}

#[macro_export]
macro_rules! create_output_dir {
    ($self: ident) => {
        if !$self.sample_output_dir.exists() {
            std::fs::create_dir_all(&$self.sample_output_dir)?;
        }
    };
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum SampleNameFormat {
    /// Capture sample name as a single word
    /// at the beginning of the file name.
    /// Consists of alphanumeric characters only.
    /// The rest of the file name is ignored.
    /// Example:
    /// - sample1_1.fastq,
    /// - sample1_2.fastq,
    /// - sample1_singleton.fastq
    /// - sample2_L001_R1.fastq
    /// Use SIMPLE_NAME_REGEX pattern -> r"(^[a-zA-Z0-9]+)"
    Simple,
    /// Capture descriptive sample name with
    /// multiple words separated by underscores or hyphens.
    /// Example:
    /// - genus_species_1.fastq,
    /// - genus_species_2.fastq,
    /// - genus_species_singleton.fastq
    /// - genus_species_L001_R1.fastq
    Descriptive,
    /// Define custom pattern for capturing sample name
    Custom(String),
}

impl std::str::FromStr for SampleNameFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(Self::Simple),
            "descriptive" => Ok(Self::Descriptive),
            _ => Ok(Self::Custom(s.to_string())),
        }
    }
}

impl Display for SampleNameFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Simple => write!(f, "simple"),
            Self::Descriptive => write!(f, "descriptive"),
            Self::Custom(pattern) => write!(f, "{}", pattern),
        }
    }
}

pub struct ReadAssignment<'a> {
    pub files: &'a [PathBuf],
    pub name_format: &'a SampleNameFormat,
}

impl<'a> ReadAssignment<'a> {
    pub fn new(files: &'a [PathBuf], name_format: &'a SampleNameFormat) -> Self {
        Self { files, name_format }
    }

    pub fn assign(&self) -> Vec<FastqReads> {
        let file_map = self.read();
        let mut reads = self.match_reads(file_map);
        reads.par_sort_by(|a, b| a.sample_name.cmp(&b.sample_name));
        reads
    }

    fn read(&self) -> HashMap<String, Vec<PathBuf>> {
        let file_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let (tx, rx) = std::sync::mpsc::channel();

        self.files.par_iter().for_each_with(tx, |tx, file| {
            let sample_name = self.get_sample_name(file);
            // Insert sample name or to hashmap or update the value
            tx.send((sample_name, file.to_path_buf()))
                .expect("Failed to send file path");
        });

        rx.iter().fold(file_map, |mut acc, (k, v)| {
            acc.entry(k).or_default().push(v);
            acc
        })
    }

    fn match_reads(&self, file_map: HashMap<String, Vec<PathBuf>>) -> Vec<FastqReads> {
        file_map
            .into_par_iter()
            .map(|(k, v)| {
                let mut reads = FastqReads::new();
                reads.match_all(k, &v);
                reads
            })
            .collect::<Vec<FastqReads>>()
    }

    fn get_sample_name(&self, file: &Path) -> String {
        let capture = match self.name_format {
            SampleNameFormat::Simple => re_capture_lazy!(SIMPLE_NAME_REGEX, file),
            SampleNameFormat::Descriptive => re_capture_lazy!(DESCRIPTIVE_NAME_REGEX, file),
            SampleNameFormat::Custom(pattern) => re_capture_dynamic!(pattern, file),
        };

        match capture {
            Some(capture_text) => {
                let mut capture = capture_text.to_string();
                if self.name_format == &SampleNameFormat::Descriptive {
                    self.pop_last_character(&mut capture);
                }
                capture
            }
            None => {
                let file_name = file
                    .file_name()
                    .expect("Failed to get file name")
                    .to_str()
                    .expect("Failed to convert file name to string");
                eprintln!("Failed to capture sample name from file: {}", file_name);
                file_name.to_string()
            }
        }
    }

    fn pop_last_character(&self, sample_name: &mut String) {
        sample_name.pop();
    }
}

/// FastqReads struct to hold read files
/// and its metadata.
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize, Default)]
pub struct FastqReads {
    pub sample_name: String,
    /// Read 1 metadata
    /// Enforce the use of Option to allow
    /// for the absence of read 1 file.
    /// If read 1 is absent, the value is None.
    pub read_1: Option<FileMetadata>,
    /// Read 2 metadata
    /// Ignore read 2 if it is absent.
    /// It won't be printed in the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_2: Option<FileMetadata>,
    /// Singletons metadata
    /// Ignore singletons if it is absent.
    /// The same as read 2, it won't be printed in the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub singletons: Option<FileMetadata>,
}

impl FastqReads {
    /// Create a new FastqReads instances
    pub fn new() -> Self {
        Self {
            sample_name: String::new(),
            read_1: None,
            read_2: None,
            singletons: None,
        }
    }

    /// Match all reads to the FastqReads struct
    pub fn match_all(&mut self, sample_name: String, reads: &[PathBuf]) {
        self.check_reads(reads, reads.len());
        self.sample_name = sample_name;
        reads.iter().for_each(|r| {
            self.match_read(r);
        });
    }

    /// Match reads if the reads are known
    pub fn match_define_reads(&mut self, sample_name: String, read1: &Path, read2: Option<&Path>) {
        self.read_1 = Some(self.metadata(read1));
        if let Some(r2) = read2 {
            self.read_2 = Some(self.metadata(r2));
        }
        self.sample_name = sample_name;
    }

    /// Get read 1 file path
    /// Return empty path if read 1 is absent.
    pub fn get_read1(&self) -> PathBuf {
        if let Some(meta) = &self.read_1 {
            meta.canonicalize()
        } else {
            PathBuf::new()
        }
    }

    /// Get read 2 file path
    /// Show warning if read 2 is absent.
    pub fn get_read2(&self) -> Option<PathBuf> {
        if let Some(meta) = &self.read_2 {
            let path = meta.canonicalize();
            Some(path)
        } else {
            let msg = format!(
                "\nRead 2 file not found for {}. \
                Proceeding with single end reads\n",
                self.sample_name
            );
            log::warn!("{}", msg.yellow());
            None
        }
    }

    /// Get singletons file path
    /// Ignore singletons if it is absent.
    pub fn get_singleton(&self) -> Option<PathBuf> {
        if let Some(meta) = &self.singletons {
            let path = meta.canonicalize();
            Some(path)
        } else {
            None
        }
    }

    fn match_read(&mut self, file_path: &Path) {
        if re_match!(READ1_REGEX, file_path) {
            self.read_1 = Some(self.metadata(file_path));
        } else if re_match!(READ2_REGEX, file_path) {
            self.read_2 = Some(self.metadata(file_path));
        } else {
            self.singletons = Some(self.metadata(file_path));
        }
    }

    fn metadata(&self, file_path: &Path) -> FileMetadata {
        let mut metadata = FileMetadata::new();
        metadata.get(file_path);
        metadata
    }

    fn check_reads(&self, reads: &[PathBuf], len: usize) {
        let help_msg = format!(
            "Please, check sample name format. \
        Make sure it matches to the right format. \
        You can specify the format using the {} argument \
        or use regex to match the sample name using the {} argument",
            "--sample-name".green(),
            "--re-sample".green()
        );
        let too_many_reads = "Too many reads found".red();
        let sample_founds = reads
            .iter()
            .enumerate()
            .map(|(i, r)| format!("{}: {:?}\n", i + 1, r.display()))
            .collect::<String>();
        assert!(len > 0, "No reads found. {}", help_msg.red());
        assert!(
            len <= 3,
            "{}. {}.\nProblematic reads:\n{}",
            too_many_reads,
            help_msg,
            sample_founds
        );
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum RawReadStatus {
    /// Assume paired-end reads
    Complete,
    CompleteWithSingleton,
    Warning(RawReadWarningKind),
    Error(RawReadErrorKind),
}

impl Default for RawReadStatus {
    fn default() -> Self {
        Self::Complete
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum RawReadErrorKind {
    MissingRead1,
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum RawReadWarningKind {
    MissingRead2,
    MismatchHash,
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum ReadChecksumStatus {
    Match,
    Mismatch(String),
}

pub struct ReadChecker {
    pub sample_name: String,
    pub completeness_status: RawReadStatus,
    pub checksum_status: Vec<ReadChecksumStatus>,
}

impl ReadChecker {
    pub fn new(sample_name: &str) -> Self {
        Self {
            sample_name: sample_name.to_string(),
            completeness_status: RawReadStatus::default(),
            checksum_status: Vec::new(),
        }
    }

    pub fn check(&mut self, reads: &FastqReads, ignore_checksum: bool) {
        self.check_completeness(reads);
        if !ignore_checksum {
            self.checksum(reads);
        }
    }

    /// Check if the reads are error-free
    /// It is error-free if the reads are complete
    /// and the checksums match for all reads.
    pub fn is_ok(&self) -> bool {
        self.completeness_status == RawReadStatus::Complete
            && self
                .checksum_status
                .iter()
                .all(|s| s == &ReadChecksumStatus::Match)
    }

    pub fn has_warnings(&self) -> bool {
        matches!(&self.completeness_status, RawReadStatus::Warning(_))
    }

    pub fn has_errors(&self) -> bool {
        matches!(&self.completeness_status, RawReadStatus::Error(_))
    }

    fn check_completeness(&mut self, reads: &FastqReads) {
        let has_read1 = reads.read_1.is_some();
        let has_read2 = reads.read_2.is_some();
        let has_singleton = reads.singletons.is_some();

        if has_read1 && has_read2 {
            self.completeness_status = RawReadStatus::Complete;
        } else if has_singleton {
            self.completeness_status = RawReadStatus::CompleteWithSingleton;
        } else if has_read1 && !has_read2 {
            self.completeness_status = RawReadStatus::Warning(RawReadWarningKind::MissingRead2);
        } else {
            self.completeness_status = RawReadStatus::Error(RawReadErrorKind::MissingRead1);
        }
    }

    fn checksum(&mut self, reads: &FastqReads) {
        if let Some(read1) = &reads.read_1 {
            self.checksum_status.push(self.check_hash(read1));
        }

        if let Some(read2) = &reads.read_2 {
            self.checksum_status.push(self.check_hash(read2));
        }

        if let Some(singletons) = &reads.singletons {
            self.checksum_status.push(self.check_hash(singletons));
        }
    }

    fn check_hash(&self, read: &FileMetadata) -> ReadChecksumStatus {
        let file_path = read.parent_dir.join(&read.file_name);
        let checksum = ChecksumType::Sha256;
        let hash = checksum
            .sha256(&file_path)
            .expect("Failed to generate hash");
        if hash == read.sha256 {
            ReadChecksumStatus::Match
        } else {
            ReadChecksumStatus::Mismatch(file_path.display().to_string())
        }
    }
}
