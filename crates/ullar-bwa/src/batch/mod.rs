use std::path::{Path, PathBuf};

use ullar::{
    helper::files::FileFinder,
    types::{
        SupportedFormats,
        reads::{FastqReads, ReadAssignment, SampleNameFormat},
    },
};

use crate::bwa::subprocess::BwaMem;

pub struct BatchBwaAlign {
    pub dir: PathBuf,
    pub reference: PathBuf,
    pub recursive: bool,
    pub sample_name_format: SampleNameFormat,
    pub output: PathBuf,
}

impl BatchBwaAlign {
    pub fn new(
        dir: PathBuf,
        sample_name_format: SampleNameFormat,
        reference: PathBuf,
        recursive: bool,
        output: PathBuf,
    ) -> Self {
        BatchBwaAlign {
            dir,
            reference,
            recursive,
            sample_name_format,
            output,
        }
    }

    pub fn builder() -> BatchBwaAlignBuilder {
        BatchBwaAlignBuilder::default()
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
        let processed_samples = 0;
        for read in reads {
            println!("Aligning sample: {}", read.sample_name);
            self.run_bwa(&read);
            let processed_samples = processed_samples + 1;
            println!("Completed {}/{} samples.", processed_samples, total_samples);
        }
    }

    fn run_bwa(&self, read: &FastqReads) {
        let bwa_mem = BwaMem::builder()
            .reference_path(&self.reference)
            .query_read1(read.get_read1())
            .query_read2(read.get_read2())
            .output_path(self.output.join(format!("{}.sam", read.sample_name)))
            .output_format("bam")
            .build()
            .expect("Failed to build BWA MEM command");
        bwa_mem.align().expect("Failed to run BWA MEM");
    }

    fn find_reads(&self) -> Vec<FastqReads> {
        let files = FileFinder::new(&self.dir, &SupportedFormats::Fastq)
            .find(self.recursive)
            .expect("Failed to find read files");
        ReadAssignment::new(&files, &self.sample_name_format).assign()
    }
}

#[derive(Default)]
pub struct BatchBwaAlignBuilder {
    dir: Option<PathBuf>,
    reference: Option<PathBuf>,
    recursive: bool,
    sample_name_format: SampleNameFormat,
    output: Option<PathBuf>,
}

impl BatchBwaAlignBuilder {
    pub fn dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.dir = Some(dir.as_ref().to_path_buf());
        self
    }

    pub fn sample_name_format(mut self, format: &str) -> Self {
        self.sample_name_format = format
            .parse::<SampleNameFormat>()
            .unwrap_or(SampleNameFormat::Descriptive);
        self
    }

    pub fn reference<P: AsRef<Path>>(mut self, reference: P) -> Self {
        self.reference = Some(reference.as_ref().to_path_buf());
        self
    }

    pub fn output<P: AsRef<Path>>(mut self, output: P) -> Self {
        self.output = Some(output.as_ref().to_path_buf());
        self
    }

    pub fn recursive(mut self, yes: bool) -> Self {
        self.recursive = yes;
        self
    }

    pub fn build(self) -> Result<BatchBwaAlign, &'static str> {
        Ok(BatchBwaAlign::new(
            self.dir.ok_or("dir is required")?,
            self.sample_name_format,
            self.reference.ok_or("reference is required")?,
            self.recursive,
            self.output.ok_or("output is required")?,
        ))
    }
}
