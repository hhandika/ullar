use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use colored::Colorize;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    cli::commands::common::CommonInitArgs,
    types::reads::{FastqReads, ReadChecker},
};

#[derive(Debug, Serialize, Default, Deserialize)]
pub struct FastqInput {
    /// Input directory
    pub input_dir: PathBuf,
    /// Total samples input
    pub sample_counts: usize,
    /// Total files input
    pub file_counts: usize,
    /// How reads are assigned to samples
    pub read_assignment: ReadAssignmentStrategy,
}

impl FastqInput {
    /// Initialize a new ConfigSummary instance
    pub fn new(
        input_dir: &Path,
        sample_counts: usize,
        file_counts: usize,
        read_assignment: ReadAssignmentStrategy,
    ) -> Self {
        Self {
            input_dir: input_dir.to_path_buf(),
            sample_counts,
            file_counts,
            read_assignment,
        }
    }

    pub fn log_summary(&self) {
        log::info!("{:18}: {}", "Total samples", self.sample_counts);
        log::info!("{:18}: {}", "Total files", self.file_counts);
        log::info!("{:18}: {:?}", "Read assignment", self.read_assignment);
    }
}

pub struct FastqConfigCheck {
    /// Total samples input
    pub total_samples: usize,
    /// Samples passed the check
    pub passed_samples: usize,
    /// Samples with warnings
    pub warning_samples: usize,
    /// Samples failed the check
    pub failed_samples: usize,
}

impl FastqConfigCheck {
    /// Initialize a new ConfigCheck instance
    pub fn new(total_samples: usize) -> Self {
        Self {
            total_samples,
            passed_samples: 0,
            warning_samples: 0,
            failed_samples: 0,
        }
    }

    pub fn check_fastq(&mut self, samples: &[FastqReads], ignore_checksum: bool) {
        let status = self.check_config(samples, ignore_checksum);
        self.passed_samples = status.iter().filter(|s| s.is_ok()).count();
        self.warning_samples = status.iter().filter(|s| s.has_warnings()).count();
        self.failed_samples = status.iter().filter(|s| s.has_errors()).count();
    }

    pub fn is_config_ok(&self) -> bool {
        self.passed_samples == self.total_samples
    }

    fn check_config(&self, samples: &[FastqReads], ignore_checksum: bool) -> Vec<ReadChecker> {
        let (tx, rx) = mpsc::channel();
        samples.par_iter().for_each_with(tx, |tx, sample| {
            let mut status = ReadChecker::new(&sample.sample_name);
            status.check(sample, ignore_checksum);
            tx.send(status).expect("Failed to send status");
        });

        rx.iter().collect()
    }

    pub fn log_status(&self) {
        log::info!("{}", "Config check summary".cyan());
        log::info!("{:18}: {}", "Total samples", self.total_samples);
        let ok_text = format!("{:18}: {}", "Pass", self.passed_samples);
        log::info!("{}", ok_text.green());

        if self.warning_samples > 0 {
            log::info!("{:18}: {}", "Warning".yellow(), self.warning_samples);
        }

        if self.failed_samples > 0 {
            log::info!("{:18}: {}", "Fail", self.failed_samples);
        }
    }
}

/// Read assignment strategy
/// determines how reads are assigned to samples
/// based on the sample name format
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum ReadAssignmentStrategy {
    /// Assign reads based on a regular expression
    Regex { format: String },
    /// Custom Regex
    CustomRegex { pattern: String },
    /// Assign reads based on a character split
    /// with a separator and separator counts
    CharacterSplit { separator: char, length: usize },
}

impl Default for ReadAssignmentStrategy {
    fn default() -> Self {
        Self::Regex {
            format: "descriptive".to_string(),
        }
    }
}

impl ReadAssignmentStrategy {
    pub fn from_arg(common: &CommonInitArgs) -> Self {
        if let Some(regex) = &common.re_sample {
            Self::CustomRegex {
                pattern: regex.to_string(),
            }
        } else {
            Self::match_strategy(common.separator, common.length, &common.sample_name)
        }
    }

    fn match_strategy(separator: Option<char>, length: Option<usize>, regex: &str) -> Self {
        match separator {
            Some(sep) => Self::CharacterSplit {
                separator: sep,
                length: length.unwrap_or(1),
            },
            None => Self::Regex {
                format: regex.to_string(),
            },
        }
    }
}
