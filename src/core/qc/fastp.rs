//! Run fastp for quality control.

use std::{
    error::Error,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::helper::reads::FastqReads;

pub const FASTP_EXE: &str = "fastp";

const FASTP_HTML: &str = "fastp.html";
const FASTP_JSON: &str = "fastp.json";
const FASTP_LOG: &str = "fastp.log";
const FASTP_REPORT_DIR: &str = "reports";

/// Run fastp for quality control
pub struct FastpRunner<'a> {
    /// Sample to process
    sample: &'a FastqReads,
    /// Output directory
    pub sample_output_dir: PathBuf,
    /// User specified fastp parameters
    /// Input as space separated string
    pub optional_params: Option<&'a str>,
}

impl<'a> FastpRunner<'a> {
    /// Create a new FastpRunner instance
    pub fn new(
        sample: &'a FastqReads,
        output_dir: &'a Path,
        optional_params: Option<&'a str>,
    ) -> Self {
        FastpRunner {
            sample,
            sample_output_dir: output_dir.join(&sample.sample_name),
            optional_params,
        }
    }

    /// Run fastp
    pub fn run(&mut self) -> Result<Option<FastpReport>, Box<dyn Error>> {
        let read1 = self.get_read1();
        let read2 = self.get_read2();
        self.create_output_dir()?;
        let output = self.execute_fastp(&read1, read2.as_deref())?;

        self.check_spades_success(&output)?;

        Ok(None)
    }

    fn create_output_dir(&self) -> Result<(), Box<dyn Error>> {
        std::fs::create_dir_all(&self.sample_output_dir)?;
        Ok(())
    }

    fn check_spades_success(&self, output: &Output) -> Result<Option<FastpReport>, Box<dyn Error>> {
        if output.status.success() {
            let report = FastpReport::new(&self.sample_output_dir);
            report.create(&output)?;
            return Ok(Some(report));
        } else {
            println!();
            io::stdout().write_all(&output.stdout).unwrap();
            io::stdout().write_all(&output.stderr).unwrap();
            return Err("Error running fastp".into());
        }
    }

    fn get_read1(&self) -> PathBuf {
        if let Some(meta) = &self.sample.read_1 {
            let path = meta.parent_dir.join(&meta.file_name);
            path.to_path_buf()
                .canonicalize()
                .expect("Read 1 file not found")
        } else {
            panic!("Read 1 file not found")
        }
    }

    fn get_read2(&self) -> Option<PathBuf> {
        if let Some(meta) = &self.sample.read_2 {
            let path = meta
                .parent_dir
                .join(&meta.file_name)
                .canonicalize()
                .expect("Read 2 file not found");
            Some(path)
        } else {
            log::warn!("Read 2 file not found");
            log::warn!("Proceeding with single end reads");
            None
        }
    }

    fn execute_fastp(&self, read1: &Path, read2: Option<&Path>) -> Result<Output, Box<dyn Error>> {
        let output_read1 = self
            .sample_output_dir
            .join(read1.file_name().expect("Read 1 file not found"));

        let mut cmd = Command::new(FASTP_EXE);

        cmd.arg("-i").arg(read1);
        cmd.arg("-o").arg(&output_read1);
        if let Some(input_r2) = read2 {
            let output_read2 = self
                .sample_output_dir
                .join(input_r2.file_name().expect("Read 2 file not found"));
            cmd.arg("-I").arg(input_r2);
            cmd.arg("-O").arg(&output_read2);
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

pub struct FastpReport<'a> {
    pub output_dir: &'a Path,
    pub html: PathBuf,
    pub json: PathBuf,
    pub log: PathBuf,
}

impl<'a> FastpReport<'a> {
    pub fn new(output_dir: &'a Path) -> Self {
        FastpReport {
            output_dir,
            html: PathBuf::from(FASTP_HTML),
            json: PathBuf::from(FASTP_JSON),
            log: PathBuf::from(FASTP_LOG),
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