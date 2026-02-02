//! Phase reads using sample-specific reference sequences.
//!
//! Use `ullar-segul transpose` to create individual-level reference sequences.

use colored::Colorize;
use segul::helper::{finder::SeqFileFinder, types::InputFmt};
use std::{fs, path::PathBuf};
use ullar_bam::{finder::files::BamFileFinder, types::BamFormat};

use crate::samtools::phase::SamtoolsPhase;

pub struct BatchPhaseBam {
    /// Input bam
    pub input_dir: PathBuf,
    /// Reference sequence directory
    pub reference_dir: PathBuf,
    /// Output directory
    pub output_dir: PathBuf,
    /// Reference is locus alignments
    /// That requires transposing sequences to
    /// create individual-level references.
    /// Reference file will be named as `<sample_name>.fas`
    /// And the contents will be all locus sequences
    /// found in the all alignment files.
    pub is_locus_aligned_refs: bool,
    /// Find reads recursively in the read directory
    pub recursive: bool,
    bam_format: BamFormat,
    reference_format: InputFmt,
}

impl BatchPhaseBam {
    pub fn new<P: AsRef<std::path::Path>>(input_dir: P) -> Self {
        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            reference_dir: PathBuf::new(),
            output_dir: PathBuf::new(),
            is_locus_aligned_refs: false,
            recursive: false,
            bam_format: BamFormat::Bam,
            reference_format: InputFmt::Auto,
        }
    }

    pub fn reference_dir<P: AsRef<std::path::Path>>(&mut self, reference_dir: P) -> &mut Self {
        self.reference_dir = reference_dir.as_ref().to_path_buf();
        self
    }

    pub fn output_dir<P: AsRef<std::path::Path>>(&mut self, output_dir: P) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        self
    }

    pub fn is_locus_aligned_refs(&mut self, yes: bool) -> &mut Self {
        self.is_locus_aligned_refs = yes;
        self
    }

    pub fn recursive(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    pub fn dry_run(&self) {
        let bam_files = self.find_bam_files();
        if bam_files.is_empty() {
            log::warn!("{}", "No BAM files found to phase reads.".yellow().bold());
            return;
        }
        let total_files = bam_files.len();
        log::info!("Found {} BAM files to phase reads.", total_files);
        let references = self.find_references();
        if references.is_empty() {
            log::warn!(
                "{}",
                "No reference files found for phasing.".yellow().bold()
            );
        } else {
            log::info!(
                "{}",
                format!("Found {} reference files for phasing.", references.len())
                    .green()
                    .bold()
            );
        }

        for bam_file in bam_files {
            let sample_name = self.get_sample_name(&bam_file);
            println!("Found sample: {}", sample_name);
            println!("  BAM file: {:?}", bam_file);
        }

        log::info!(
            "{}",
            "Dry run completed. No phasing was performed."
                .green()
                .bold()
        );
    }

    pub fn phase(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bam_files = self.find_bam_files();
        if bam_files.is_empty() {
            log::warn!("{}", "No BAM files found to phase reads.".yellow().bold());
            return Ok(());
        }
        let references = self.find_references();
        if references.is_empty() {
            log::warn!(
                "{}",
                "No reference files found for phasing.".yellow().bold()
            );
            return Ok(());
        } else {
            log::info!(
                "{}",
                format!("Found {} reference files for phasing.", references.len())
                    .green()
                    .bold()
            );
        }
        let total_files = bam_files.len();
        log::info!("Found {} BAM files to phase reads.", total_files);
        fs::create_dir_all(&self.output_dir)?;
        let mut processed_files = 0;
        for bam_file in bam_files {
            let sample_name = self.get_sample_name(&bam_file);
            let msg = format!("Phasing reads for sample: {}", sample_name);
            log::info!("{}", msg.cyan().bold());
            let output_subdir = self.get_output_dir(&sample_name);
            let prefix = self.get_prefix(&output_subdir, &sample_name);
            let mut phaser = SamtoolsPhase::new(bam_file);
            phaser.prefix(prefix);

            match phaser.phase() {
                Ok(_) => log::info!(
                    "{}",
                    format!("Successfully phased reads for sample: {}", sample_name)
                        .green()
                        .bold()
                ),
                Err(e) => log::error!(
                    "{}",
                    format!("Error phasing reads for sample: {}: {}", sample_name, e)
                        .red()
                        .bold()
                ),
            }
            processed_files += 1;
            log::info!(
                "{}",
                format!("Processed {}/{} samples.", processed_files, total_files)
                    .blue()
                    .bold()
            );
        }
        Ok(())
    }

    fn find_bam_files(&self) -> Vec<PathBuf> {
        let finder = BamFileFinder::new(&self.input_dir, self.recursive, self.bam_format);
        match finder.find() {
            Ok(files) => files,
            Err(e) => {
                log::error!("Error finding BAM files: {}", e);
                vec![]
            }
        }
    }

    fn get_output_dir(&self, sample_name: &str) -> PathBuf {
        let output_subdir = self.output_dir.join(sample_name);
        fs::create_dir_all(&output_subdir).expect("Failed to create output subdirectory");
        output_subdir
    }

    fn get_sample_name(&self, file_path: &PathBuf) -> String {
        file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown_sample")
            .to_string()
    }

    fn get_prefix(&self, output_dir: &PathBuf, sample_name: &str) -> PathBuf {
        output_dir.join(sample_name)
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
}
