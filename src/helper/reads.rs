//! Match file path to FASTQ reads

use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    helper::regex::{DESCRIPTIVE_NAME_REGEX, READ1_REGEX, READ2_REGEX, SIMPLE_NAME_REGEX},
    re_capture, re_capture_dynamic, re_capture_lazy, re_match,
};

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

    pub fn assign(&self) -> BTreeMap<String, FastqReads> {
        let file_map = self.read();
        let reads = self.match_reads(file_map);
        reads
    }

    fn read(&self) -> HashMap<String, Vec<PathBuf>> {
        let mut file_map = HashMap::new();
        for file in self.files {
            let sample_name = self.get_sample_name(file);
            file_map
                .entry(sample_name)
                .or_insert_with(Vec::new)
                .push(file.clone());
        }
        file_map
    }

    fn match_reads(&self, file_map: HashMap<String, Vec<PathBuf>>) -> BTreeMap<String, FastqReads> {
        file_map
            .into_iter()
            .map(|(k, v)| {
                let mut reads = FastqReads::new();
                reads.match_all(&v);
                (k, reads)
            })
            .collect::<BTreeMap<String, FastqReads>>()
    }

    fn get_sample_name(&self, file: &PathBuf) -> String {
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

#[allow(dead_code)]
pub struct FastqReads {
    pub parent_path: PathBuf,
    pub read_1: String,
    pub read_2: Option<String>,
    pub singletons: Option<String>,
}

#[allow(dead_code)]
impl FastqReads {
    pub fn new() -> Self {
        Self {
            parent_path: PathBuf::new(),
            read_1: String::new(),
            read_2: None,
            singletons: None,
        }
    }

    pub fn match_all(&mut self, reads: &[PathBuf]) {
        self.check_reads(reads.len());
        reads.iter().for_each(|r| {
            self.parent_path = r.parent().unwrap_or(Path::new(".")).to_path_buf();
            self.match_read(r);
        });
    }

    fn match_read(&mut self, file_path: &Path) {
        let file_name = file_path
            .file_name()
            .expect("Failed to get file name")
            .to_str()
            .expect("Failed to convert file name to string");
        if re_match!(READ1_REGEX, file_path) {
            self.read_1 = file_name.to_string();
        } else if re_match!(READ2_REGEX, file_path) {
            self.read_2 = Some(file_name.to_string());
        } else {
            self.singletons = Some(file_name.to_string());
        }
    }

    fn check_reads(&self, len: usize) {
        let help_msg = "Please, check sample name format. \
        Make sure it matches to the right format. \
        You can specify the format using the --sample-name flag.";
        assert!(len > 0, "No reads found. {}", help_msg);
        assert!(len <= 3, "Too many reads found. {}.", help_msg);
    }
}
