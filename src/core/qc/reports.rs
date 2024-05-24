//! Data structure to generate Fastp report

use std::{error::Error, path::PathBuf, process::Output};

use walkdir::WalkDir;

use super::fastp::Fastp;

const FASTP_HTML: &str = "fastp.html";
const FASTP_JSON: &str = "fastp.json";
const FASTP_LOG: &str = "fastp.log";
const FASTP_REPORT_DIR: &str = "reports";

#[derive(Debug, Clone)]
pub struct FastpReport {
    pub fastp_data: Fastp,
    pub sample_name: String,
    pub html: PathBuf,
    pub json: PathBuf,
    pub log: PathBuf,
}

impl FastpReport {
    pub fn new(fastp_data: Fastp, sample_name: &str) -> Self {
        FastpReport {
            fastp_data,
            sample_name: sample_name.to_string(),
            html: PathBuf::from(FASTP_HTML),
            json: PathBuf::from(FASTP_JSON),
            log: PathBuf::from(FASTP_LOG),
        }
    }

    pub fn create(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        self.write_log(output)?;
        self.organize()?;
        Ok(())
    }

    /// Clean up the output directory by removing all empty directories
    pub fn finalize(&self) {
        WalkDir::new(&self.fastp_data.output_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            // Filter only empty directories
            .filter(|e| {
                e.path().is_dir()
                    && e.path()
                        .read_dir()
                        .expect("Failed finding directory")
                        .next()
                        .is_none()
            })
            .for_each(|e| std::fs::remove_dir(&e.path()).expect("Failed removing directory"));
    }

    fn write_log(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.log, &output.stderr)?;
        Ok(())
    }

    fn organize(&self) -> Result<(), Box<dyn Error>> {
        let report_dir = self.fastp_data.output_dir.join(FASTP_REPORT_DIR);
        std::fs::create_dir_all(&report_dir)?;
        std::fs::rename(&self.html, report_dir.join(FASTP_HTML))?;
        std::fs::rename(&self.json, report_dir.join(FASTP_JSON))?;

        Ok(())
    }
}
