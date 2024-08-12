//! Run fastp for quality control.

use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use colored::Colorize;
use indicatif::ProgressBar;

use crate::{
    check_read1_exists, create_output_dir,
    helper::common::{self, PrettyHeader},
    parse_override_args,
    types::reads::FastqReads,
};

use super::reports::CleanReadReport;

pub const FASTP_EXE: &str = "fastp";

/// Run fastp for quality control
pub struct FastpRunner<'a> {
    /// Sample to process
    sample: &'a FastqReads,
    /// Output directory
    pub sample_output_dir: PathBuf,
    /// User specified fastp parameters
    /// Input as space separated string
    pub override_args: Option<&'a str>,
}

impl<'a> FastpRunner<'a> {
    /// Create a new FastpRunner instance
    pub fn new(
        sample: &'a FastqReads,
        output_dir: &'a Path,
        override_args: Option<&'a str>,
    ) -> Self {
        FastpRunner {
            sample,
            sample_output_dir: output_dir.join(&sample.sample_name),
            override_args,
        }
    }

    /// Run fastp
    pub fn run(&mut self) -> Result<CleanReadReport, Box<dyn Error>> {
        let decorator = self.print_header();
        let read1 = self.sample.get_read1();
        check_read1_exists!(self, read1);
        let read2 = self.sample.get_read2();
        self.print_input_summary(&read1, read2.as_deref());
        create_output_dir!(self);
        let spinner = common::init_spinner();
        spinner.set_message("Cleaning reads");
        let mut fastp = Fastp::new(&self.sample_output_dir);
        let output = fastp.execute(&read1, read2.as_deref(), self.override_args);

        match output {
            Ok(output) => self.create_report(&output, fastp, &spinner, &decorator),
            Err(e) => {
                spinner.finish_with_message(format!("{} Failed to clean reads\n", "✘".red()));
                Err(e)
            }
        }
    }

    fn create_report(
        &self,
        output: &Output,
        fastp_data: Fastp,
        spinner: &ProgressBar,
        decorator: &PrettyHeader,
    ) -> Result<CleanReadReport, Box<dyn Error>> {
        let reports = self.check_success(output, fastp_data, spinner);
        match reports {
            Ok(report) => {
                self.print_output_summary(&report);
                decorator.get_sample_footer();
                Ok(report)
            }
            Err(e) => {
                decorator.get_sample_footer();
                Err(e)
            }
        }
    }

    fn check_success(
        &self,
        output: &Output,
        fastp_data: Fastp,
        spinner: &ProgressBar,
    ) -> Result<CleanReadReport, Box<dyn Error>> {
        if output.status.success() {
            spinner.set_message(format!("Creating report for {}", self.sample.sample_name));
            let report = CleanReadReport::new(fastp_data, &self.sample.sample_name);
            report.create(output)?;
            report.finalize();
            spinner.finish_with_message(format!("{} Finished cleaning reads\n", "✔".green()));
            Ok(report)
        } else {
            spinner.finish_with_message(format!("{} Failed to clean reads\n", "✘".red()));
            let err = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::info!("\n{}", err);
            log::info!("{}", stdout);
            Err("Failed to clean reads".into())
        }
    }

    fn print_header(&self) -> PrettyHeader {
        let mut decorator = PrettyHeader::new();
        let header = decorator.get_sample_header(&self.sample.sample_name);
        log::info!("{}", header);
        decorator
    }

    fn print_input_summary(&self, read1: &Path, read2: Option<&Path>) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Read 1", self.get_file_name(read1));
        if let Some(read2) = read2 {
            log::info!("{:18}: {}", "Read 2", self.get_file_name(read2));
        }
        log::info!("{:18}: AUTO-DETECT\n", "Adapter");
    }

    fn get_file_name(&self, path: &Path) -> String {
        path.file_name()
            .expect("Failed to get file name")
            .to_string_lossy()
            .to_string()
    }

    fn print_output_summary(&self, report: &CleanReadReport) {
        log::info!("{}", "Output".cyan());
        log::info!(
            "{:18}: {}",
            "Directory",
            report.fastp_data.output_dir.display()
        );
        log::info!("{:18}: {}", "HTML", report.html.display());
        log::info!("{:18}: {}", "JSON", report.json.display());
        log::info!("{:18}: {}\n", "Log", report.log.display());
    }
}

#[derive(Debug, Clone)]
pub struct Fastp {
    pub output_dir: PathBuf,
    pub read1_filename: String,
    pub read2_filename: Option<String>,
}

impl Fastp {
    pub fn new(output_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
            read1_filename: String::new(),
            read2_filename: None,
        }
    }

    fn execute(
        &mut self,
        input_read1: &Path,
        input_read2: Option<&Path>,
        override_args: Option<&str>,
    ) -> Result<Output, Box<dyn Error>> {
        self.get_read1_filename(input_read1);
        let output_read1 = self.output_dir.join(self.read1_filename.as_str());
        let mut cmd = Command::new(FASTP_EXE);

        cmd.arg("-i").arg(input_read1);
        if let Some(r2) = input_read2 {
            self.get_read2_filename(r2);
            let output_read2 = self.output_dir.join(
                self.read2_filename
                    .as_ref()
                    .expect("Failed to get read 2 filename"),
            );
            cmd.arg("-I").arg(r2);
            cmd.arg("-O").arg(&output_read2);
        }
        cmd.arg("-o").arg(&output_read1);

        if let Some(params) = override_args {
            self.build_custom_params(&mut cmd, params);
        }

        Ok(cmd.output()?)
    }

    fn get_read1_filename(&mut self, input_read1: &Path) {
        self.read1_filename = input_read1
            .file_name()
            .expect("Failed to get read 1 file name")
            .to_string_lossy()
            .to_string();
    }

    fn get_read2_filename(&mut self, input_read2: &Path) {
        self.read2_filename = Some(
            input_read2
                .file_name()
                .expect("Failed to get read 2 file name")
                .to_string_lossy()
                .to_string(),
        );
    }

    fn build_custom_params(&self, cmd: &mut Command, params: &str) {
        parse_override_args!(cmd, params);
    }
}
