use std::path::{Path, PathBuf};

use colored::Colorize;
use ullar_bam::{finder::files::BamFileFinder, types::BamFormat};

use crate::samtools::index::SamtoolsIndex;

pub struct BatchIndexBams {
    pub input_dir: PathBuf,
    pub recursive: bool,
    pub format: BamFormat,
}

impl BatchIndexBams {
    pub fn new<P: AsRef<Path>>(input_dir: P) -> Self {
        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            recursive: false,
            format: BamFormat::Bam,
        }
    }

    pub fn format(&mut self, format: BamFormat) -> &mut Self {
        self.format = format;
        self
    }

    pub fn recursive(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    pub fn dry_run(&self) {
        // Implementation for dry run to list BAM files to be indexed
        let bam_files = self.find_bam_files();
        if bam_files.is_empty() {
            log::warn!("{}", "No BAM files found to index.".yellow().bold());
            return;
        }
        let total_files = bam_files.len();
        log::info!("Found {} BAM files to index.", total_files);
    }

    pub fn index(&self) {
        let bam_files = self.find_bam_files();
        if bam_files.is_empty() {
            log::warn!("{}", "No BAM files found to index.".yellow().bold());
            return;
        }
        log::info!(
            "{}",
            format!("Indexing {} BAM files...", bam_files.len())
                .green()
                .bold()
        );
        let total_files = bam_files.len();
        let mut processed_files = 0;

        for bam in bam_files {
            let indexer = SamtoolsIndex::new(&bam);
            match indexer.create_index() {
                Ok(_) => log::info!("{}", format!("Indexed BAM file: {}", bam.display()).green()),
                Err(e) => log::error!(
                    "{}",
                    format!("Failed to index BAM file {}: {}", bam.display(), e).red()
                ),
            }
            processed_files += 1;
            log::info!(
                "{}",
                format!("Progress: {}/{} completed", processed_files, total_files)
                    .blue()
                    .bold()
            );
        }
    }

    fn find_bam_files(&self) -> Vec<PathBuf> {
        let finder = BamFileFinder::new(&self.input_dir, self.recursive, self.format);
        match finder.find() {
            Ok(files) => files,
            Err(e) => {
                log::error!("{}", format!("Error finding BAM files: {}", e).red());
                vec![]
            }
        }
    }
}
