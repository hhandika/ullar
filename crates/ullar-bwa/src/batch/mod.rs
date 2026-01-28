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

use crate::bwa::mem::BwaMem;

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

    pub fn run(&self) {
        let reads = self.find_reads();
        let total_samples = reads.len();
        println!("Found {} samples to align.", total_samples);
        let mut processed_samples = 0;
        fs::create_dir_all(&self.output).expect("Failed to create output directory");
        for read in reads {
            let msg = format!("Processing sample: {}", read.sample_name);
            println!("{}", msg.cyan().bold());
            let output_path = self.get_output_path(&read.sample_name);
            self.run_bwa(&read, &output_path);
            processed_samples += 1;
            let progress = format!("Completed {}/{} samples.", processed_samples, total_samples);
            println!("{} {}", "✓", progress.green().bold());
        }
    }

    fn run_bwa(&self, read: &FastqReads, output_path: &Path) {
        let mut bwa_mem = BwaMem::new(&read.sample_name);
        bwa_mem
            .reference_path(&self.reference)
            .query_read1(read.get_read1())
            .query_read2(read.get_read2())
            .output_path(output_path)
            .output_format(&self.output_format)
            .threads(self.threads);
        bwa_mem.align().expect("BWA MEM alignment failed");
    }

    fn find_reads(&self) -> Vec<FastqReads> {
        let files = FileFinder::new(&self.dir, &SupportedFormats::Fastq)
            .find(self.recursive)
            .expect("Failed to find read files");
        println!("Found {} read files.", files.len());
        ReadAssignment::new(&files, &self.sample_name_format).assign()
    }

    fn get_output_path(&self, sample_name: &str) -> PathBuf {
        self.output
            .join(format!("{}.{}", sample_name, self.output_format))
    }
}
