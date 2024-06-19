use std::sync::mpsc;

use colored::Colorize;
use rayon::prelude::*;

use crate::helper::reads::{FastqReads, ReadChecker};

pub mod cleaned_reads;
pub mod raw_reads;

pub struct ConfigCheck {
    /// Total samples input
    pub total_samples: usize,
    /// Samples passed the check
    pub passed_samples: usize,
    /// Samples with warnings
    pub warning_samples: usize,
    /// Samples failed the check
    pub failed_samples: usize,
}

impl ConfigCheck {
    /// Initialize a new ConfigCheck instance
    pub fn new(total_samples: usize) -> ConfigCheck {
        ConfigCheck {
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
