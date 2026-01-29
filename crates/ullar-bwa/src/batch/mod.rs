use colored::Colorize;

use std::{
    fs,
    path::{Path, PathBuf},
};
use ullar::{
    helper::files::FileFinder,
    types::{
        SupportedFormats,
        reads::{FastqReads, ReadAssignment, SampleNameFormat},
    },
};
use ullar_fastq::files::reader::FastqReader;

use crate::{
    bwa::{mem::BwaMem, types::BwaRunStatus},
    samtools::index::SamtoolsIndex,
};

pub struct BatchBwaAlign {
    pub dir: PathBuf,
    pub reference: PathBuf,
    pub recursive: bool,
    pub output_format: String,
    pub sample_name_format: SampleNameFormat,
    pub threads: usize,
    pub output: PathBuf,
}

impl BatchBwaAlign {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        BatchBwaAlign {
            dir: dir.as_ref().to_path_buf(),
            reference: PathBuf::new(),
            recursive: false,
            output_format: "bam".to_string(),
            sample_name_format: SampleNameFormat::default(),
            threads: 4,
            output: PathBuf::new(),
        }
    }

    pub fn reference<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.reference = p.as_ref().to_path_buf();
        self
    }

    pub fn output<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.output = p.as_ref().to_path_buf();
        self
    }

    pub fn recursive(mut self, yes: bool) -> Self {
        self.recursive = yes;
        self
    }

    pub fn threads(mut self, n: usize) -> Self {
        self.threads = n;
        self
    }

    pub fn dry_run(&self) {
        let reads = self.find_reads();
        println!("Found {} samples to align.", reads.len());
        for read in reads {
            println!("Found sample: {}", read.sample_name);
            println!("  Read 1: {:?}", read.get_read1());
            println!("  Read 2: {:?}", read.get_read2());
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let reads = self.find_reads();
        let total_samples = reads.len();
        log::info!("Found {} samples to align.", total_samples);
        let mut processed_samples = 0;
        fs::create_dir_all(&self.output).expect("Failed to create output directory");
        for read in reads {
            let msg = format!("Processing sample: {}", read.sample_name);
            log::info!("{}", msg.cyan().bold());
            let output_path = self.get_output_path(&read.sample_name)?;
            let status = self.run_bwa(&read, &output_path)?;
            if status == BwaRunStatus::Success {
                self.log_align_completed(&read.sample_name);
                if self.output_format.to_lowercase() == "bam" {
                    self.index_bam(&output_path);
                }
            } else if let BwaRunStatus::Failure(err_msg) = status {
                self.log_align_error(&read.sample_name, &err_msg);
            }
            processed_samples += 1;
            let progress = format!("Completed {}/{} samples.", processed_samples, total_samples);
            log::info!("{} {}", "✓", progress.green().bold());
        }
        Ok(())
    }

    fn run_bwa(
        &self,
        read: &FastqReads,
        output_path: &Path,
    ) -> Result<BwaRunStatus, Box<dyn std::error::Error>> {
        let mut bwa_mem = BwaMem::new(&read.sample_name);
        bwa_mem
            .reference_path(&self.reference)
            .query_read1(read.get_read1())
            .query_read2(read.get_read2())
            .output_path(output_path)
            .read_group(&self.get_read_group(read))
            .output_format(&self.output_format)
            .threads(self.threads);
        bwa_mem.align()
    }

    fn index_bam(&self, bam_path: &Path) {
        if !bam_path.exists() {
            log::warn!(
                "BAM file {} does not exist. Skipping indexing.",
                bam_path.display()
            );
            return;
        }
        let mut samtools_index = SamtoolsIndex::new(bam_path);
        samtools_index.output_path(bam_path.with_extension("bai"));
        samtools_index
            .create_index()
            .expect("Failed to create BAM index");
    }

    fn get_read_group(&self, read: &FastqReads) -> String {
        let file_path = read.get_read1();
        let mut reader = FastqReader::new(&file_path).expect("Failed to create FASTQ reader");
        let read_groud = reader
            .get_read_group()
            .expect("Failed to extract Read Group from FASTQ header");
        read_groud
    }

    fn find_reads(&self) -> Vec<FastqReads> {
        let files = FileFinder::new(&self.dir, &SupportedFormats::Fastq)
            .find(self.recursive)
            .expect("Failed to find read files");
        log::info!("Found {} read files.", files.len());
        ReadAssignment::new(&files, &self.sample_name_format).assign()
    }

    fn get_output_path(&self, sample_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = self.output.join(sample_name);
        fs::create_dir_all(&output_dir)?;
        let output_path = output_dir
            .join(sample_name)
            .with_extension(&self.output_format);
        Ok(output_path)
    }

    fn log_align_completed(&self, sample_name: &str) {
        log::info!(
            "{} {}",
            "✓".green().bold(),
            format!("Alignment completed for sample: {}", sample_name)
        );
    }

    fn log_align_error(&self, sample_name: &str, error_msg: &str) {
        log::error!(
            "{} {}",
            "✗".red().bold(),
            format!(
                "Alignment failed for sample: {}. Error: {}",
                sample_name, error_msg
            )
        );
    }
}
