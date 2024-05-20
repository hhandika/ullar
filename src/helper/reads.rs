//! Match file path to FASTQ reads

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    helper::regex::{DESCRIPTIVE_NAME_REGEX, READ1_REGEX, READ2_REGEX, SIMPLE_NAME_REGEX},
    re_capture, re_capture_dynamic, re_capture_lazy, re_match,
};

use super::files::FileMetadata;

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
            acc.entry(k).or_insert_with(Vec::new).push(v);
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
        self.check_reads(reads.len());
        self.sample_name = sample_name;
        reads.iter().for_each(|r| {
            self.match_read(r);
        });
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

    fn check_reads(&self, len: usize) {
        let help_msg = "Please, check sample name format. \
        Make sure it matches to the right format. \
        You can specify the format using the --sample-name flag.";
        assert!(len > 0, "No reads found. {}", help_msg);
        assert!(len <= 3, "Too many reads found. {}.", help_msg);
    }
}
