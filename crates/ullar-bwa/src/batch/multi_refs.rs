//! Bwa Align to multiple reference individual-specific references.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::ChildStdout,
};

use colored::Colorize;
use segul::helper::{finder::SeqFileFinder, types::InputFmt};
use ullar_fastq::{
    files::reads::get_read_group_ilumina,
    types::reads::{FastqReads, SampleNameFormat},
};
use ullar_samtools::samtools::{fixmate::SamtoolsFixmate, sort::SamtoolsSort};

use crate::bwa::{index::BwaIndex, mem::BwaMem, types::BwaFormat};

pub struct BatchBwaAlignMultiRefs {
    pub dir: PathBuf,
    pub reference_dir: PathBuf,
    pub recursive: bool,
    pub bwa_executable: String,
    pub output_dir: PathBuf,
    sample_name_format: SampleNameFormat,
    reference_format: InputFmt,
    output_format: BwaFormat,
}

impl BatchBwaAlignMultiRefs {
    pub fn new<P: AsRef<std::path::Path>>(dir: P) -> Self {
        BatchBwaAlignMultiRefs {
            dir: dir.as_ref().to_path_buf(),
            reference_dir: PathBuf::new(),
            recursive: false,
            bwa_executable: "bwa-mem2".to_string(),
            output_dir: PathBuf::new(),
            sample_name_format: SampleNameFormat::default(),
            reference_format: InputFmt::Auto,
            output_format: BwaFormat::Bam,
        }
    }

    pub fn reference_dir<P: AsRef<std::path::Path>>(&mut self, reference_dir: P) -> &mut Self {
        self.reference_dir = reference_dir.as_ref().to_path_buf();
        self
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    pub fn bwa_executable<S: Into<String>>(&mut self, executable: S) -> &mut Self {
        self.bwa_executable = executable.into();
        self
    }

    pub fn output_dir<P: AsRef<std::path::Path>>(&mut self, output_dir: P) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&self.output_dir).expect("Unable to create output directory");
        self
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "{}",
            "Starting batch BWA alignment to multiple references..."
                .cyan()
                .bold()
        );
        let reads = self.find_reads();
        log::info!("Found {} samples to align.", reads.len());
        let references = self.find_references();
        log::info!("Found {} reference files.", references.len());
        let matching_refs = self.match_reads_to_reference(&references, &reads);
        if matching_refs.len() != references.len() {
            log::warn!(
                "{}",
                "Some references do not have matching reads and will be skipped."
                    .yellow()
                    .bold()
            );
        }
        let sample_counts = matching_refs.len();
        let mut processed_samples = 0;
        for (sample_name, ref_path) in references {
            if let Some(fq_reads) = reads.get(&sample_name) {
                self.index_ref(&ref_path)?;
                let stdout = self.align_reads_to_reference(&ref_path, fq_reads)?;
                let sorted_stdout = self.sort_by_sample_name(stdout, &sample_name)?;
                let fixed_stdout = self.fix_mate(sorted_stdout, &sample_name)?;
                match self.sort_by_coordinate(fixed_stdout, &sample_name) {
                    Ok(_) => {
                        self.log_success(&sample_name);
                    }
                    Err(e) => {
                        self.log_error(&sample_name, &e.to_string());
                    }
                }
                processed_samples += 1;
                log::info!(
                    "{}",
                    format!("Processed {}/{} samples.", processed_samples, sample_counts)
                        .green()
                        .bold()
                );
            } else {
                log::warn!(
                    "No reads found for sample: {}. Skipping alignment.",
                    sample_name
                );
            }
        }
        Ok(())
    }

    fn align_reads_to_reference(
        &self,
        ref_path: &Path,
        reads: &FastqReads,
    ) -> Result<ChildStdout, Box<dyn std::error::Error>> {
        let msg = format!("Aligning sample: {}", reads.sample_name);
        log::info!("{}", msg.cyan().bold());
        let read_group = get_read_group_ilumina(reads);
        let mut bwa_mem = BwaMem::new(&reads.sample_name);
        bwa_mem
            .reference_path(ref_path)
            .query_read1(reads.get_read1())
            .query_read2(reads.get_read2())
            .read_group(&read_group?)
            .output_format(self.output_format);

        let stdout = bwa_mem.align_piped()?;
        Ok(stdout)
    }

    fn index_ref(&self, ref_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "{}",
            format!("Indexing reference: {}", ref_path.display())
                .cyan()
                .bold()
        );
        let mut bwa = BwaIndex::new(ref_path);
        bwa.index_prefix(ref_path)
            .set_executable(self.bwa_executable.parse().unwrap_or_default());
        bwa.index();
        Ok(())
    }

    fn sort_by_sample_name(
        &self,
        stdout: ChildStdout,
        sample_name: &str,
    ) -> Result<ChildStdout, Box<dyn std::error::Error>> {
        log::info!(
            "{}",
            format!("Sorting alignments by name for sample: {}", sample_name)
                .cyan()
                .bold()
        );
        let by_name = true;
        let mut samtools = SamtoolsSort::from_bwa_stdout(stdout, sample_name);
        let sorted_stdout = samtools.sort_piped_by(by_name)?;
        Ok(sorted_stdout)
    }

    fn sort_by_coordinate(
        &self,
        stdout: ChildStdout,
        sample_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "{}",
            format!(
                "Sorting alignments by coordinate for sample: {}",
                sample_name
            )
            .cyan()
            .bold()
        );
        let mut samtools = SamtoolsSort::from_bwa_stdout(stdout, sample_name);
        samtools.output_path(&self.get_output_path(sample_name)?);
        samtools.to_bam_sorted()?;
        Ok(())
    }

    fn fix_mate(
        &self,
        stdout: ChildStdout,
        sample_name: &str,
    ) -> Result<ChildStdout, Box<dyn std::error::Error>> {
        log::info!(
            "{}",
            format!("Fixing mate information for sample: {}", sample_name)
                .cyan()
                .bold()
        );
        let samtools = SamtoolsFixmate::fixmate_from_stdout_piped(stdout, 4)?;
        Ok(samtools)
    }

    // We use sample name to map reads to reference sequences
    fn find_reads(&self) -> HashMap<String, FastqReads> {
        log::info!("{}", "Finding reads in the input directory...");
        let read_files = ullar_fastq::files::find_and_assign_reads(
            &self.dir,
            &self.sample_name_format,
            self.recursive,
        );
        read_files
            .iter()
            .map(|r| (r.sample_name.to_string(), r.clone()))
            .collect()
    }

    fn match_reads_to_reference(
        &self,
        ref_paths: &[(String, PathBuf)],
        reads: &HashMap<String, FastqReads>,
    ) -> Vec<PathBuf> {
        let mut matching_refs: Vec<PathBuf> = Vec::with_capacity(ref_paths.len());
        let mut missing_reads: Vec<String> = Vec::new();
        for (sample_name, ref_path) in ref_paths {
            if reads.contains_key(sample_name) {
                matching_refs.push(ref_path.to_path_buf());
            } else {
                log::warn!("No reads found for sample: {}", sample_name,);
                missing_reads.push(sample_name.to_string());
            }
        }
        // Write missing references to file
        if !missing_reads.is_empty() {
            self.write_missing_reads(&missing_reads);
        }
        matching_refs
    }

    fn write_missing_reads(&self, missing_reads: &[String]) {
        let missing_file = self.output_dir.join("missing_references.txt");
        fs::write(&missing_file, missing_reads.join("\n"))
            .expect("Unable to write missing references file");
        log::warn!(
            "List of references without matching reads written to {}",
            missing_file.display()
        );
    }

    fn get_output_path(&self, sample_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = self.output_dir.join(sample_name);
        fs::create_dir_all(&output_dir)?;
        let output_path = output_dir
            .join(sample_name)
            .with_extension(&self.output_format.extension());
        Ok(output_path)
    }

    // Return a vector of (sample_name, reference_path)
    fn find_references(&self) -> Vec<(String, PathBuf)> {
        let ref_paths = SeqFileFinder::new(&self.reference_dir).find(&self.reference_format);
        ref_paths
            .iter()
            .filter_map(|p| {
                if let Some(file_stem) = p.file_stem().and_then(|s| s.to_str()) {
                    Some((file_stem.to_string(), p.to_path_buf()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn log_success(&self, sample_name: &str) {
        log::info!(
            "{} {}",
            "✓".green().bold(),
            format!("Processing completed for sample: {}", sample_name)
        );
    }

    fn log_error(&self, sample_name: &str, error_msg: &str) {
        log::error!(
            "{} {}",
            "✗".red().bold(),
            format!(
                "Processing failed for sample: {}. Error: {}",
                sample_name, error_msg
            )
        );
    }
}
