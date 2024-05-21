//! Run fastp for quality control.

use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub const FASTP_EXE: &str = "fastp";

const FASTP_HTML: &str = "fastp.html";
const FASTP_JSON: &str = "fastp.json";
const FASTP_LOG: &str = "fastp.log";
const FASTP_REPORT_DIR: &str = "reports";

/// Run fastp for quality control
pub struct FastpRunner<'a> {
    /// Read 1 input file
    pub input_r1: &'a Path,
    /// Read 2 input file (if paired-end)
    pub input_r2: Option<&'a Path>,
    /// Output directory
    pub output_dir: &'a Path,
    /// User specified fastp parameters
    /// Input as space separated string
    pub optional_params: Option<&'a str>,
}

impl<'a> FastpRunner<'a> {
    /// Create a new FastpRunner instance
    pub fn new(
        input_r1: &'a Path,
        input_r2: Option<&'a Path>,
        output_dir: &'a Path,
        optional_params: Option<&'a str>,
    ) -> Self {
        FastpRunner {
            input_r1,
            input_r2,
            output_dir,
            optional_params,
        }
    }

    /// Run fastp
    pub fn run(&self) -> Result<Option<FastpReport>, Box<dyn Error>> {
        let output = self.execute_fastp()?;

        if self.is_success(&output) {
            let report = FastpReport::new(self.output_dir);
            report.create(&output)?;
            return Ok(Some(report));
        }

        Ok(None)
    }

    fn is_success(&self, output: &Output) -> bool {
        output.status.success()
    }

    fn execute_fastp(&self) -> Result<Output, Box<dyn Error>> {
        let mut cmd = Command::new(FASTP_EXE);

        cmd.arg("-i").arg(self.input_r1);
        if let Some(input_r2) = self.input_r2 {
            cmd.arg("-I").arg(input_r2);
        }

        if let Some(params) = self.optional_params {
            self.build_custom_params(&mut cmd, params);
        }

        Ok(cmd.output()?)
    }

    fn build_custom_params(&self, cmd: &mut Command, params: &str) {
        params.split_whitespace().for_each(|param| {
            cmd.arg(param);
        });
    }
}

pub(crate) struct FastpReport<'a> {
    output_dir: &'a Path,
    html: PathBuf,
    json: PathBuf,
    log: PathBuf,
}

impl<'a> FastpReport<'a> {
    pub fn new(output_dir: &'a Path) -> Self {
        FastpReport {
            output_dir,
            html: output_dir.join(FASTP_HTML),
            json: output_dir.join(FASTP_JSON),
            log: output_dir.join(FASTP_LOG),
        }
    }

    fn create(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        self.write_log(output)?;
        self.organize()?;
        Ok(())
    }

    fn write_log(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.log, &output.stderr)?;
        Ok(())
    }

    fn organize(&self) -> Result<(), Box<dyn Error>> {
        let report_dir = self.output_dir.join(FASTP_REPORT_DIR);
        std::fs::create_dir_all(&report_dir)?;

        std::fs::rename(&self.html, report_dir.join(FASTP_HTML))?;
        std::fs::rename(&self.json, report_dir.join(FASTP_JSON))?;

        Ok(())
    }
}
