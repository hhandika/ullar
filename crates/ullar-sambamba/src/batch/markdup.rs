use colored::Colorize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use ullar_bam::{finder::files::BamFileFinder, types::BamFormat};

use crate::sambamba::markdup::SambambaMarkDup;

pub const DEFAULT_MARKDUP_DIR: &str = "bam_markeddup";

pub struct BatchMarkDup {
    pub executable: Option<String>,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub recursive: bool,
    pub threads: usize,
    pub remove_duplicates: bool,
    pub compression_level: Option<u8>,
    pub override_options: Option<String>,
}

impl BatchMarkDup {
    pub fn new<P: AsRef<std::path::Path>>(input_dir: P) -> Self {
        Self {
            executable: None,
            input_dir: input_dir.as_ref().to_path_buf(),
            output_dir: PathBuf::from(DEFAULT_MARKDUP_DIR),
            recursive: false,
            threads: 4,
            remove_duplicates: false,
            compression_level: None,
            override_options: None,
        }
    }

    pub fn output_dir<P: AsRef<std::path::Path>>(mut self, p: P) -> Self {
        self.output_dir = p.as_ref().to_path_buf();
        self
    }

    pub fn executable(mut self, exe: &str) -> Self {
        self.executable = Some(exe.to_string());
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

    pub fn remove_duplicates(mut self, yes: bool) -> Self {
        self.remove_duplicates = yes;
        self
    }

    pub fn compression_level(mut self, level: u8) -> Self {
        let clamped_level = if level > 9 { 9 } else { level };
        self.compression_level = Some(clamped_level);
        self
    }

    pub fn override_options(mut self, options: &str) -> Self {
        self.override_options = Some(options.to_string());
        self
    }

    pub fn dry_run(&self) {
        log::info!("Batch MarkDup Configuration:");
        if let Some(ref exe) = self.executable {
            log::info!("  Sambamba Executable: {}", exe);
        } else {
            log::info!("  Sambamba Executable: sambamba (default)");
        }
        log::info!("  Input Directory: {}", self.input_dir.display());
        log::info!("  Output Directory: {}", self.output_dir.display());
        log::info!("  Recursive: {}", self.recursive);
        log::info!("  Threads: {}", self.threads);
        log::info!("  Remove Duplicates: {}", self.remove_duplicates);
        if let Some(level) = self.compression_level {
            log::info!("  Compression Level: {}", level);
        } else {
            log::info!("  Compression Level: Default");
        }
        if let Some(ref options) = self.override_options {
            log::info!("  Override Options: {}", options);
        } else {
            log::info!("  Override Options: None");
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bam_files = self.find_bam_files();
        fs::create_dir_all(&self.output_dir)?;
        let sample_count = bam_files.len();
        log::info!(
            "Found {} BAM files to process in {}",
            sample_count,
            self.input_dir.display()
        );
        let mut processed_samples = 0;
        for bam_file in bam_files {
            let file_stem = self.get_file_stem(&bam_file);
            let msg = format!("Processing sample: {}", file_stem);
            log::info!("{}", msg.blue().bold());
            let output_dir = self.get_output_dir(&file_stem);
            let output_bam = self.get_output_file(&output_dir, &file_stem);
            let mut markdup = SambambaMarkDup::new(self.executable.as_deref(), &file_stem);
            markdup
                .input_bam(&bam_file)
                .output_bam(&output_bam)
                .remove_duplicates(self.remove_duplicates)
                .threads(self.threads);

            if let Some(level) = self.compression_level {
                markdup.compression_level(level);
            }

            if let Some(ref options) = self.override_options {
                markdup.override_options(options);
            }

            markdup.execute()?;
            log::info!("Finished processing: {}", bam_file.display());
            processed_samples += 1;
            let progress = format!("{} Completed: {}/{}", "✓", processed_samples, sample_count);
            log::info!("{}", progress.green().bold());
        }
        Ok(())
    }

    fn find_bam_files(&self) -> Vec<PathBuf> {
        let finder = BamFileFinder::new(&self.input_dir, self.recursive, BamFormat::Bam);
        match finder.find() {
            Ok(files) => files,
            Err(e) => {
                log::error!("Error finding BAM files: {}", e);
                vec![]
            }
        }
    }

    fn get_output_dir(&self, file_stem: &str) -> PathBuf {
        let output_subdir = self.output_dir.join(file_stem);
        fs::create_dir_all(&output_subdir).expect("Failed to create output subdirectory");
        output_subdir
    }

    fn get_file_stem(&self, file_path: &PathBuf) -> String {
        file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string()
    }

    fn get_output_file(&self, output_dir: &Path, file_stem: &str) -> PathBuf {
        output_dir.join(file_stem).with_extension("bam")
    }
}
