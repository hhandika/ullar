//! Data structure to generate Fastp report

use std::{error::Error, path::PathBuf, process::Output};

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

    fn write_log(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.log, &output.stderr)?;
        Ok(())
    }

    fn organize(&self) -> Result<(), Box<dyn Error>> {
        let report_dir = self.fastp_data.output_dir.join(FASTP_REPORT_DIR);
        std::fs::create_dir_all(&report_dir)?;
        println!("{}", report_dir.display());
        std::fs::rename(&self.html, report_dir.join(FASTP_HTML))?;
        std::fs::rename(&self.json, report_dir.join(FASTP_JSON))?;

        Ok(())
    }
}
