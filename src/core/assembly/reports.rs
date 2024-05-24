//! Data structure for Assembly report

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

const SPADES_CONTIGS: &str = "contigs.fasta";
const SPADES_SCAFFOLDS: &str = "scaffolds.fasta";
const SPADES_REPORT: &str = "spades_report.html";
const SPADES_LOG: &str = "spades.log";
const CONTIG_SUFFIX: &str = "-contigs";
const CONTIG_EXTENSION: &str = "fasta";

pub struct SpadeReports {
    pub sample_name: String,
    pub output_dir: PathBuf,
    pub contigs: PathBuf,
    pub scaffolds: PathBuf,
    pub report: PathBuf,
    pub log: PathBuf,
}

impl SpadeReports {
    pub fn new(sample_name: &str, output_dir: &Path) -> SpadeReports {
        SpadeReports {
            sample_name: sample_name.to_string(),
            output_dir: output_dir.to_path_buf(),
            contigs: output_dir.join(SPADES_CONTIGS),
            scaffolds: output_dir.join(SPADES_SCAFFOLDS),
            report: output_dir.join(SPADES_REPORT),
            log: output_dir.join(SPADES_LOG),
        }
    }

    pub fn remove_intermediates(&self) -> Result<(), Box<dyn Error>> {
        WalkDir::new(&self.output_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !self.is_essential_spades_file(e.path()))
            .for_each(|e| self.remove(&e.path()));
        Ok(())
    }

    // We rename contigs.fasta to sample_name-contigs.fasta
    pub fn finalize(&mut self) {
        let contigs = self.output_dir.join(SPADES_CONTIGS);
        let new_contigs = self
            .output_dir
            .join(format!("{}{}", self.sample_name, CONTIG_SUFFIX))
            .with_extension(CONTIG_EXTENSION);

        let rename = std::fs::rename(&contigs, &new_contigs);
        match rename {
            Ok(_) => {
                log::info!(
                    "\n{} {}\n",
                    "Contigs file was renamed to",
                    new_contigs.display()
                );
                self.contigs = new_contigs;
            }
            Err(e) => {
                log::error!("Failed to rename contigs file: {}", e);
                log::error!("Contigs file will be saved as: {}", contigs.display());
            }
        }
    }

    fn is_essential_spades_file(&self, file: &Path) -> bool {
        // We don't want to remove the output directory
        if file.is_dir() {
            file.ends_with(&self.output_dir)
        } else {
            file.ends_with(&self.contigs)
                || file.ends_with(&self.scaffolds)
                || file.ends_with(&self.report)
                || file.ends_with(&self.log)
        }
    }

    fn remove(&self, entry: &Path) {
        if entry.is_file() {
            std::fs::remove_file(entry)
                .expect(&format!("Failed to remove file {}", entry.display()));
        }

        // We remove the directory and its contents
        if entry.is_dir() {
            std::fs::remove_dir_all(entry)
                .expect(&format!("Failed to remove directory {}", entry.display()));
        }
    }
}
