use crate::bwa::errors::validate_bwa_inputs;
use crate::bwa::types::{BwaFormat, BwaRunStatus};
use crate::samtools::sort::SamtoolsSort;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct BwaMem {
    pub reference_path: PathBuf,
    pub query_read1: PathBuf,
    pub query_read2: Option<PathBuf>,
    pub output_path: PathBuf,
    pub output_format: BwaFormat,
    pub read_group: String,
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
            output_format: BwaFormat::Bam,
            read_group: String::new(),
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
            "sam" => BwaFormat::Sam,
            "bam" => BwaFormat::Bam,
            _ => BwaFormat::Bam,
        };
        self.use_samtools_view = matches!(self.output_format, BwaFormat::Bam);
        self
    }

    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = threads;
        self
    }

    pub fn read_group(&mut self, rg: &str) -> &mut Self {
        self.read_group = rg.to_string();
        self
    }

    pub fn align(&self) -> Result<BwaRunStatus, Box<dyn std::error::Error>> {
        if self.use_samtools_view {
            self.align_to_samtools_bam()
        // } else {
        //     self.align_to_bam()
        } else {
            Err("Only BAM output via samtools view is currently supported".into())
        }
    }

    // fn align_to_bam(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut bwa = self.get_bwa_command();
    //     bwa.arg("-o").arg(&self.output_path);
    //     self.write_output(&mut bwa)
    // }

    /// Align reads using BWA mem and output to BAM using samtools view
    fn align_to_samtools_bam(&self) -> Result<BwaRunStatus, Box<dyn std::error::Error>> {
        self.validate_inputs()?;
        let log_file = File::create("bwa.log")?;
        let mut bwa = self.get_bwa_command();
        let mut bwa_child = bwa
            .stdout(Stdio::piped())
            .stderr(Stdio::from(log_file))
            .spawn()?;

        let bwa_stdout = bwa_child
            .stdout
            .take()
            .ok_or("Failed to capture BWA stdout")?;
        let mut sam =
            SamtoolsSort::new(Some(bwa_stdout), &self.sample_name).output_path(&self.output_path);
        sam.to_bam()?;

        let bwa_output = bwa_child.wait_with_output()?;
        if !bwa_output.status.success() {
            let stderr = String::from_utf8_lossy(&bwa_output.stderr);
            return Err(format!("BWA mem command failed: {}", stderr).into());
        }

        Ok(BwaRunStatus::Success)
    }

    pub fn get_bwa_command(&self) -> Command {
        let mut bwa = Command::new("bwa-mem2.avx2");

        bwa.arg("mem")
            .arg("-t")
            .arg(self.get_threads().to_string())
            .arg("-R")
            .arg(&self.read_group)
            .arg(&self.reference_path)
            .arg(&self.query_read1);

        if let Some(read2) = &self.query_read2 {
            bwa.arg(read2);
        }

        bwa
    }

    fn validate_inputs(&self) -> Result<(), Box<dyn std::error::Error>> {
        validate_bwa_inputs(&self.query_read1, &self.query_read2)?;
        Ok(())
    }

    // fn write_output(&self, output: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    //     let output: Output = output.output()?;
    //     if !output.status.success() {
    //         let stderr = String::from_utf8_lossy(&output.stderr);
    //         return Err(format!("BWA mem command failed: {}", stderr).into());
    //     }

    //     let mut bam_file = fs::File::create(&self.output_path)?;
    //     let mut bam_writer = BamWriter::new(&mut bam_file);

    //     let bwa_stdout = BufReader::new(&output.stdout[..]);
    //     let mut sam_reader = sam::io::Reader::new(bwa_stdout);

    //     // Read and PARSE header
    //     let header = sam_reader.read_header()?;
    //     bam_writer.write_header(&header)?;

    //     // Process EACH record: read -> PARSE -> write
    //     for result in sam_reader.records() {
    //         let bam_record = bam::Record::from_sam_record(&header, &result?)?;
    //         bam_writer.write_record(&header, &record)?;
    //     }

    //     bam_writer.finish()?;
    //     Ok(())
    // }

    fn get_threads(&self) -> usize {
        if self.threads > 0 {
            return self.threads;
        }
        let sys_threads = num_cpus::get();
        let threads = sys_threads / 2;
        if threads > 4 { threads } else { sys_threads }
    }
}
