//! Run fastp for quality control.

use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use colored::Colorize;
use indicatif::ProgressBar;

use crate::helper::{
    reads::FastqReads,
    utils::{self, PrettyHeader},
};

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
    pub fn run(&mut self) -> Result<FastpReport, Box<dyn Error>> {
        let decorator = self.print_header();
        let read1 = self.get_read1();

        if !read1.exists() {
            let msg = format!(
                "\nRead 1 file not found for {}. Skipping it!\n",
                self.sample.sample_name
            );
            log::error!("{}", msg.red());
            decorator.get_sample_footer();
            return Err("Read 1 file not found".into());
        }

        let read2 = self.get_read2();
        self.print_input_summary(&read1, read2.as_deref());
        self.create_output_dir()?;
        let spinner = utils::init_spinner();
        spinner.set_message("Cleaning reads");
        let mut fastp = Fastp::new(&self.sample_output_dir);
        let output = fastp.execute(&read1, read2.as_deref(), self.optional_params.as_deref())?;

        let reports = self.check_success(&output, fastp, &spinner)?;
        match reports {
            Some(report) => {
                self.print_output_summary(&report);
                decorator.get_sample_footer();
                Ok(report)
            }
            None => {
                decorator.get_sample_footer();
                Err("Failed to clean reads".into())
            }
        }
    }

    fn create_output_dir(&self) -> Result<(), Box<dyn Error>> {
        std::fs::create_dir_all(&self.sample_output_dir)?;
        Ok(())
    }

    fn check_success(
        &self,
        output: &Output,
        fastp_data: Fastp,
        spinner: &ProgressBar,
    ) -> Result<Option<FastpReport>, Box<dyn Error>> {
        if output.status.success() {
            spinner.finish_with_message(format!("{} Finished cleaning reads\n", "✔".green()));
            let report = FastpReport::new(fastp_data);
            report.create(&output)?;
            return Ok(Some(report));
        } else {
            spinner.finish_with_message(format!("{} Failed to clean reads\n", "✘".red()));
            let err = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::info!("\n{}", err);
            log::info!("{}", stdout);
            // We return None here because
            // we don't want to stop the process for the next samples
            return Ok(None);
        }
    }

    fn get_read1(&self) -> PathBuf {
        if let Some(meta) = &self.sample.read_1 {
            let path = meta.parent_dir.join(&meta.file_name);
            path.to_path_buf()
                .canonicalize()
                .expect("Failed to get read 1 path")
        } else {
            PathBuf::new()
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
            let msg = format!(
                "\nRead 2 file not found for {}. \
                Proceeding with single end reads\n",
                self.sample.sample_name
            );
            log::warn!("{}", msg.yellow());
            None
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

    fn print_output_summary(&self, report: &FastpReport) {
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
pub struct FastpReport {
    pub fastp_data: Fastp,
    pub html: PathBuf,
    pub json: PathBuf,
    pub log: PathBuf,
}

impl FastpReport {
    pub fn new(fastp_data: Fastp) -> Self {
        FastpReport {
            fastp_data,
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
        let report_dir = self.fastp_data.output_dir.join(FASTP_REPORT_DIR);
        std::fs::create_dir_all(&report_dir)?;

        std::fs::rename(&self.html, report_dir.join(FASTP_HTML))?;
        std::fs::rename(&self.json, report_dir.join(FASTP_JSON))?;

        Ok(())
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
        optional_params: Option<&str>,
    ) -> Result<Output, Box<dyn Error>> {
        self.get_read1_filename(input_read1);
        let output_read1 = self.output_dir.join(self.read1_filename.as_str());

        let mut cmd = Command::new(FASTP_EXE);

        cmd.arg("-i").arg(input_read1);
        cmd.arg("-o").arg(&output_read1);
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

        if let Some(params) = optional_params {
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
        params.split_whitespace().for_each(|param| {
            cmd.arg(param);
        });
    }
}
