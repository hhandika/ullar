use crate::bwa::errors::validate_bwa_inputs;
use crate::bwa::types::BwaOutputFormat;
use crate::samtools::view::SamtoolsView;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub struct BwaMem {
    pub reference_path: PathBuf,
    pub query_read1: PathBuf,
    pub query_read2: Option<PathBuf>,
    pub output_path: PathBuf,
    pub output_format: BwaOutputFormat,
    pub sample_name: String,
    pub use_samtools_view: bool,
    pub threads: usize,
}

impl BwaMem {
    pub fn new(sample_name: &str) -> Self {
        BwaMem {
            reference_path: PathBuf::new(),
            query_read1: PathBuf::new(),
            query_read2: None,
            output_path: PathBuf::new(),
            output_format: BwaOutputFormat::Bam,
            sample_name: sample_name.to_string(),
            use_samtools_view: true,
            threads: 2,
        }
    }

    pub fn reference_path<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.reference_path = p.as_ref().to_path_buf();
        self
    }

    pub fn query_read1<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.query_read1 = p.as_ref().to_path_buf();
        self
    }

    pub fn query_read2<P: AsRef<Path>>(&mut self, p: Option<P>) -> &mut Self {
        if let Some(path) = p {
            self.query_read2 = Some(path.as_ref().to_path_buf());
        }
        self
    }

    pub fn output_path<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn output_format(&mut self, format: &str) -> &mut Self {
        self.output_format = match format.to_lowercase().as_str() {
            "sam" => BwaOutputFormat::Sam,
            "bam" => BwaOutputFormat::Bam,
            _ => BwaOutputFormat::Bam,
        };
        self.use_samtools_view = matches!(self.output_format, BwaOutputFormat::Bam);
        self
    }

    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = threads;
        self
    }

    pub fn align(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.validate_inputs()?;
        let mut bwa = Command::new("bwa");

        bwa.arg("mem")
            .arg("-t")
            .arg(self.get_threads().to_string())
            .arg("-v")
            .arg("1")
            .arg(&self.reference_path)
            .arg(&self.query_read1);

        if let Some(read2) = &self.query_read2 {
            bwa.arg(read2);
        }

        if self.use_samtools_view {
            let mut bwa_child = bwa.stdout(Stdio::piped()).spawn()?;

            let bwa_stdout = bwa_child
                .stdout
                .take()
                .ok_or("Failed to capture BWA stdout")?;
            let mut sam = SamtoolsView::new(Some(bwa_stdout), &self.sample_name)
                .output_path(&self.output_path);
            sam.to_bam()?;

            let bwa_output = bwa_child.wait_with_output()?;
            if !bwa_output.status.success() {
                let stderr = String::from_utf8_lossy(&bwa_output.stderr);
                return Err(format!("BWA mem failed: {}", stderr).into());
            }
        } else {
            self.write_output(&mut bwa)?;
        }

        Ok(())
    }

    fn validate_inputs(&self) -> Result<(), Box<dyn std::error::Error>> {
        validate_bwa_inputs(&self.query_read1, &self.query_read2)?;
        Ok(())
    }

    fn write_output(&self, output: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
        let output: Output = output.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("BWA mem command failed: {}", stderr).into());
        }

        fs::write(&self.output_path, output.stdout)?;
        Ok(())
    }

    fn get_threads(&self) -> usize {
        if self.threads > 0 {
            return self.threads;
        }
        let sys_threads = num_cpus::get();
        let threads = sys_threads / 2;
        if threads > 4 { threads } else { sys_threads }
    }
}
