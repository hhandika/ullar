//! Match file path to FASTQ reads

use std::{collections::HashMap, path::PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    helper::regex::{DESCRIPTIVE_NAME_REGEX, SIMPLE_NAME_REGEX},
    re_capture, re_capture_dynamic, re_capture_lazy,
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

pub struct ReadAssignment<'a> {
    pub files: &'a [PathBuf],
    pub name_format: &'a SampleNameFormat,
    file_map: HashMap<String, Vec<PathBuf>>,
}

impl<'a> ReadAssignment<'a> {
    pub fn new(files: &'a [PathBuf], name_format: &'a SampleNameFormat) -> Self {
        Self {
            files,
            name_format,
            file_map: HashMap::new(),
        }
    }

    pub fn assign_reads(&mut self) {
        let pattern = self.get_pattern();

        for file in self.files {
            let sample_name = self.get_sample_name(file, pattern);
            self.file_map
                .entry(sample_name)
                .or_insert_with(Vec::new)
                .push(file.clone());
        }
    }

    fn get_pattern(&self) -> &'a str {
        match self.name_format {
            SampleNameFormat::Simple => SIMPLE_NAME_REGEX,
            SampleNameFormat::Descriptive => DESCRIPTIVE_NAME_REGEX,
            SampleNameFormat::Custom(pattern) => pattern,
        }
    }

    fn get_sample_name(&self, file: &PathBuf, pattern: &str) -> String {
        let capture = match self.name_format {
            SampleNameFormat::Simple => re_capture_lazy!(SIMPLE_NAME_REGEX, file),
            SampleNameFormat::Descriptive => re_capture_lazy!(DESCRIPTIVE_NAME_REGEX, file),
            SampleNameFormat::Custom(pattern) => re_capture_dynamic!(pattern, file),
        };

        match capture {
            Some(capture_text) => {
                let mut capture = capture_text.to_string();
                if self.name_format == &SampleNameFormat::Simple {
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
    pub fn new(
        parent_path: PathBuf,
        read_1: String,
        read_2: Option<String>,
        singletons: Option<String>,
    ) -> Self {
        Self {
            parent_path,
            read_1,
            read_2,
            singletons,
        }
    }
}
